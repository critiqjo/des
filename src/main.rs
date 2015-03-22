#![feature(alloc)]
// suppress warning when using Weak, downgrade, strong_count

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

extern crate rand;
use rand::distributions::IndependentSample;
use rand::distributions::{Exp, Normal, Range};

mod cpu;
mod event;
mod request;
use cpu::{Cpu, CpuState};
use event::{Event, EventType};
use request::Request;

const N_CPU: usize = 4;
const N_USERS: usize = 32;
const EASE_IN_TIME: f64 = 20.0;
const MAX_ITERS: usize = 10000;
const BUFFER_CAPACITY: usize = 1000;
const THREADPOOL_SIZE: usize = 40;
const QUANTUM: f64 = 0.5;

const REQ_SERVICE_TIME_MEAN: f64 = 2.0;
const REQ_TIMEOUT_MIN: f64 = 10.0;
const REQ_TIMEOUT_MAX: f64 = 30.0;

const THINK_TIME_MEAN: f64 = 24.0;
const THINK_TIME_STD_DEV: f64 = 8.0;
const RETRY_THINK_TIME_MEAN: f64 = 2.0;
const RETRY_THINK_TIME_STD_DEV: f64 = 1.0;

#[derive(Debug)]
struct SystemMetrics {
    time: f64,
    sum_resp_time: f64,
    n_req_proc: usize,
    n_req_timeo: usize,
    n_req_drops: usize,
}

fn proc_req(rc_cpu: Rc<RefCell<Cpu>>, rc_req: Rc<RefCell<Request>>, systime: f64) -> Event {
    {
        let mut cpu = rc_cpu.borrow_mut();
        cpu.state = CpuState::Busy(rc_req.clone());
        cpu.quantum_start = systime;
    }
    let quantum = QUANTUM; // randomized?
    let rem_serv = rc_req.borrow().remaining_service;
    if rem_serv < quantum {
        Event { _type: EventType::Departure(rc_cpu.clone()), timestamp: systime + rem_serv }
    } else {
        Event { _type: EventType::QuantumOver(rc_cpu.clone()), timestamp: systime + quantum }
    }
}

fn main() {
    let mut sys = SystemMetrics { time: 0.0, sum_resp_time: 0.0, n_req_proc: 0, n_req_timeo: 0, n_req_drops: 0 };
    let mut events = BinaryHeap::new();
    let mut rng = rand::thread_rng();
    let mut idle_cpus = Vec::with_capacity(N_CPU);
    for _ in 0..N_CPU {
        idle_cpus.push(Rc::new(RefCell::new(Cpu::new())));
    }
    let mut rbuff = VecDeque::with_capacity(BUFFER_CAPACITY);
    let mut tpool = VecDeque::with_capacity(THREADPOOL_SIZE);
    let ease_in = Range::new(0.0_f64, EASE_IN_TIME);
    let service = Exp::new(1.0/REQ_SERVICE_TIME_MEAN);
    let timeout = Range::new(REQ_TIMEOUT_MIN, REQ_TIMEOUT_MAX);

    for _ in 0..N_USERS {
        let req = Rc::new(RefCell::new(Request::new(sys.time, &ease_in, &service, &mut rng)));
        let arrival = req.borrow().arrival_time;
        let e = Event { _type: EventType::Arrival(req.clone()), timestamp: arrival };
        events.push(e);
        let e = Event { _type: EventType::Timeout(req.clone().downgrade()), timestamp: arrival + timeout.ind_sample(&mut rng) };
        events.push(e);
    }

    let mut iters = 0;
    while let Some(e) = events.pop() {
        use event::EventType::*;
        sys.time = e.timestamp;
        match e._type {
            Arrival(rc_req) => {
                println!("T={} Arrival({:?})", sys.time, &rc_req);
                if let Some(rc_cpu) = idle_cpus.pop() {
                    events.push(proc_req(rc_cpu, rc_req, sys.time));
                } else if (tpool.len() + N_CPU < THREADPOOL_SIZE) {
                    tpool.push_back(rc_req);
                } else if (rbuff.len() < BUFFER_CAPACITY) {
                    rbuff.push_back(rc_req);
                } else {
                    sys.n_req_drops += 1;
                    // TODO client retries here
                }
            },
            Departure(rc_cpu) => {
                println!("T={} Departure({:?})", sys.time, &rc_cpu);
            },
            QuantumOver(rc_cpu) => {
                println!("T={} QuantumOver({:?})", sys.time, &rc_cpu);
                {
                    let mut cpu = rc_cpu.borrow_mut();
                    let procd_time = sys.time - cpu.quantum_start;
                    cpu.total_busy_time += procd_time;
                    if let CpuState::Busy( ref rc_req ) = cpu.state {
                        rc_req.borrow_mut().remaining_service -= procd_time;
                        tpool.push_back(rc_req.clone());
                    } else {
                        panic!("Cpu should have been busy!");
                    }
                }
                events.push(proc_req(rc_cpu, tpool.pop_front().unwrap(), sys.time));
            },
            Timeout(weak_req) => match weak_req.upgrade() {
                Some(rc_req) => println!("T={} Timeout({:?})", sys.time, &rc_req),
                None => println!("T={} Timeout(None)", sys.time),
            },
        }
        iters += 1;
        if iters >= MAX_ITERS {
            break;
        }
    }
}


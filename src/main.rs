#![feature(alloc)]
// suppress warning when using Weak, downgrade, strong_count

use std::rc::Rc;
use std::rc::weak_count;
use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

extern crate rand;
use rand::ThreadRng;
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

fn event_after_proc(rc_req: Rc<RefCell<Request>>, rc_cpu: Rc<RefCell<Cpu>>, systime: f64) -> Event {
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

fn sched_new_arrival<A: IndependentSample<f64>,
                     S: IndependentSample<f64>,
                     T: IndependentSample<f64>>
                    (time: f64, arrival_sampler: &A,
                                service_sampler: &S,
                                timeout_sampler: &T,
                                rng: &mut ThreadRng) -> (Event, Event) {
    let req = Rc::new(RefCell::new(Request::new(time, arrival_sampler, service_sampler, rng)));

    let arrival_ts = req.borrow().arrival_time;
    let arrival_e = Event { _type: EventType::Arrival(req.clone()), timestamp: arrival_ts };

    let timeout_ts = arrival_ts + timeout_sampler.ind_sample(rng);
    let timeout_e = Event { _type: EventType::Timeout(req.clone().downgrade()), timestamp: timeout_ts };
    (arrival_e, timeout_e)
}

fn main() {
    let mut sys = SystemMetrics { time: 0.0, sum_resp_time: 0.0,
                                  n_req_proc: 0, n_req_timeo: 0, n_req_drops: 0 };
    let mut events = BinaryHeap::new();
    let mut rng = rand::thread_rng();
    let mut idle_cpus = Vec::with_capacity(N_CPU);
    for _ in 0..N_CPU {
        idle_cpus.push(Rc::new(RefCell::new(Cpu::new())));
    }
    let mut rbuff = VecDeque::with_capacity(BUFFER_CAPACITY);
    let mut tpool = VecDeque::with_capacity(THREADPOOL_SIZE);
    let ease_in_sampler = Range::new(0.0_f64, EASE_IN_TIME);
    let service_sampler = Exp::new(1.0/REQ_SERVICE_TIME_MEAN);
    let timeout_sampler = Range::new(REQ_TIMEOUT_MIN, REQ_TIMEOUT_MAX);
    let think_sampler = Normal::new(THINK_TIME_MEAN, THINK_TIME_STD_DEV);
    let retry_think_sampler = Normal::new(RETRY_THINK_TIME_MEAN, RETRY_THINK_TIME_STD_DEV);

    for _ in 0..N_USERS {
        let (arrival_e, timeout_e) = sched_new_arrival(sys.time,
                                                       &ease_in_sampler,
                                                       &service_sampler,
                                                       &timeout_sampler, &mut rng);
        events.push(arrival_e);
        events.push(timeout_e);
    }

    let mut iters = 0;
    while let Some(e) = events.pop() {
        use event::EventType::*;
        sys.time = e.timestamp;
        match e._type {
            Arrival(rc_req) => {
                println!("T={} Arrival({:?})", sys.time, &rc_req);
                if let Some(rc_cpu) = idle_cpus.pop() {
                    events.push(event_after_proc(rc_req, rc_cpu, sys.time));
                } else if (tpool.len() + N_CPU < THREADPOOL_SIZE) {
                    tpool.push_back(rc_req);
                } else if (rbuff.len() < BUFFER_CAPACITY) {
                    rbuff.push_back(rc_req);
                } else {
                    sys.n_req_drops += 1;
                    let (arrival_e, timeout_e) = sched_new_arrival(sys.time,
                                                                   &retry_think_sampler,
                                                                   &service_sampler,
                                                                   &timeout_sampler, &mut rng);
                    events.push(arrival_e);
                    events.push(timeout_e);
                }
            },
            Departure(rc_cpu) => {
                println!("T={} Departure({:?})", sys.time, &rc_cpu);
                {
                    let mut cpu = rc_cpu.borrow_mut();
                    let rc_req = match cpu.state {
                        CpuState::Busy(ref rc_req) => rc_req.clone(),
                        CpuState::Idle => panic!("At the time of departure, CPU should not be IDLE."),
                    };
                    if weak_count(&rc_req) != 0 { // Was not timed out
                        let (arrival_e, timeout_e) = sched_new_arrival(sys.time,
                                                                       &think_sampler,
                                                                       &service_sampler,
                                                                       &timeout_sampler, &mut rng);
                        events.push(arrival_e);
                        events.push(timeout_e);
                    }
                    cpu.total_busy_time += sys.time - cpu.quantum_start;
                    sys.sum_resp_time += sys.time - rc_req.borrow().arrival_time;
                    sys.n_req_proc += 1;
                }

                if let Some(req) = tpool.pop_front() {
                    events.push(event_after_proc(req, rc_cpu, sys.time));
                    if rbuff.len() > 0 {
                        tpool.push_back(rbuff.pop_front().unwrap());
                    }
                } else {
                    rc_cpu.borrow_mut().state = CpuState::Idle;
                    idle_cpus.push(rc_cpu);
                }
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
                events.push(event_after_proc(tpool.pop_front().unwrap(), rc_cpu, sys.time));
            },
            Timeout(weak_req) => match weak_req.upgrade() {
                Some(rc_req) => {
                    println!("T={} Timeout({:?})", sys.time, &rc_req);
                    sys.n_req_timeo += 1;
                    let (arrival_e, timeout_e) = sched_new_arrival(sys.time,
                                                                   &retry_think_sampler,
                                                                   &service_sampler,
                                                                   &timeout_sampler, &mut rng);
                    events.push(arrival_e);
                    events.push(timeout_e);
                },
                None => {}
            },
        }
        iters += 1;
        if iters >= MAX_ITERS {
            break;
        }
    }
}


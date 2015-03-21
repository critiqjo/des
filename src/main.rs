#![feature(alloc)]
// suppress warning when using Weak, downgrade, strong_count

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BinaryHeap;

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

const REQ_SERVICE_TIME_MEAN: f64 = 2.0;
const REQ_TIMEOUT_MIN: f64 = 10.0;
const REQ_TIMEOUT_MAX: f64 = 30.0;

#[derive(Debug)]
struct SystemMetrics {
    time: f64,
    sum_resp_time: f64,
    n_req_proc: usize,
    n_req_timeo: usize,
    n_req_drops: usize,
}

fn main() {
    let mut sys = SystemMetrics { time: 0.0, sum_resp_time: 0.0, n_req_proc: 0, n_req_timeo: 0, n_req_drops: 0 };
    let mut events = BinaryHeap::new();
    let mut rng = rand::thread_rng();
    let mut cpus = Vec::with_capacity(N_CPU);
    for _ in 0..N_CPU-1 {
        cpus.push(Rc::new(RefCell::new(Cpu::new())));
    }
    let cpus = cpus; //remove mutability
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

    while let Some(e) = events.pop() {
        println!("{:?}", &e);
    }
}


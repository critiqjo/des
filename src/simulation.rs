use std::rc::Rc;
use std::rc::weak_count;
use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

use rand::{ThreadRng, thread_rng};
use rand::distributions::IndependentSample;
use rand::distributions::{Exp, Normal, Range};

use cpu::{Cpu, CpuState};
use event::{Event, EventType};
use request::Request;

#[derive(RustcDecodable, RustcEncodable)]
pub struct SystemParams {
    pub n_cpu: usize,
    pub n_users: usize,
    pub ease_in_time: f64,
    pub max_iters: usize,
    pub buffer_capacity: usize,
    pub threadpool_size: usize,
    pub quantum: f64,

    pub service_time_mean: f64,
    pub req_timeout_min: f64,
    pub req_timeout_max: f64,

    pub think_time_mean: f64,
    pub think_time_std_dev: f64,
    pub retry_think_time_mean: f64,
    pub retry_think_time_std_dev: f64,
}

#[derive(Debug)]
pub struct SystemMetrics {
    pub time: f64,
    pub n_arrivals: usize,
    pub n_processed: usize, // incl. timed-out
    pub n_timedout: usize, // incl. those in-process
    pub n_dropped: usize,
    pub n_to_in_proc: usize, // timed-out but still in process
    pub sum_resp_time: f64,
    pub wt_sum_reqs_in_sys: f64, // time-weighted sum of |requests in system|
    pub total_cpu_time: f64,
}

struct ReqsInSystem {
    last_mod_ts: f64, // last modification timestamp
    count: usize,
    to_count: usize, // timed out requests in sys
}

fn process_request(rc_req: Rc<RefCell<Request>>, rc_cpu: Rc<RefCell<Cpu>>, simtime: f64, quantum: f64) -> Event {
    {
        let mut cpu = rc_cpu.borrow_mut();
        cpu.state = CpuState::Busy(rc_req.clone());
        cpu.quantum_start = simtime;
    }
    let rem_serv = rc_req.borrow().remaining_service;
    if rem_serv < quantum {
        Event::new(EventType::Departure(rc_cpu.clone()), simtime + rem_serv)
    } else {
        Event::new(EventType::QuantumOver(rc_cpu.clone()), simtime + quantum)
    }
}

fn sample_zero_lo<T: IndependentSample<f64>>(sampler: &T, rng: &mut ThreadRng) -> f64 {
    let sample = sampler.ind_sample(rng);
    if sample < 0.0 { 0.0 } else { sample }
}

pub fn run(sys: &SystemParams) -> SystemMetrics {
    let mut sim = SystemMetrics { time: 0.0, n_arrivals:0, n_processed: 0, n_timedout: 0,
                                  n_dropped: 0, n_to_in_proc: 0, sum_resp_time: 0.0,
                                  wt_sum_reqs_in_sys: 0.0, total_cpu_time: 0.0 };
    let mut reqs_in_sys = ReqsInSystem { last_mod_ts: 0.0, count: 0, to_count: 0 };
    let mut events = BinaryHeap::new();
    let mut rng = thread_rng();
    let mut cpus = Vec::with_capacity(sys.n_cpu);
    let mut idle_cpus = Vec::with_capacity(sys.n_cpu);
    let mut n_threads = 0;
    for _ in 0..sys.n_cpu {
        let cpu = Rc::new(RefCell::new(Cpu::new()));
        idle_cpus.push(cpu.clone());
        cpus.push(cpu);
    }
    let mut rbuff = VecDeque::with_capacity(sys.buffer_capacity); // Request Buffer
    let mut tpool = VecDeque::with_capacity(sys.threadpool_size); // Thread Pool
    let ease_in_sampler = Range::new(0.0_f64, sys.ease_in_time);
    let service_sampler = Exp::new(1.0/sys.service_time_mean);
    let timeout_sampler = Range::new(sys.req_timeout_min, sys.req_timeout_max);
    let think_sampler = Normal::new(sys.think_time_mean, sys.think_time_std_dev);
    let retry_think_sampler = Normal::new(sys.retry_think_time_mean, sys.retry_think_time_std_dev);

    for _ in 0..sys.n_users {
        let arrival_ts = sim.time + ease_in_sampler.ind_sample(&mut rng);
        let total_service = service_sampler.ind_sample(&mut rng);
        let timeout = timeout_sampler.ind_sample(&mut rng);
        let (arrival_e, timeout_e) = Event::new_arrival(arrival_ts, total_service, timeout);
        events.push(arrival_e);
        events.push(timeout_e);
    }

    let mut iters = 0;
    while let Some(e) = events.pop() {
        use event::EventType::*;
        sim.time = e.timestamp;
        match e._type {
            Arrival(rc_req) => {
                //println!("T={} Arrival {:?}", sim.time, rc_req.borrow());
                sim.n_arrivals += 1;
                sim.wt_sum_reqs_in_sys += (sim.time - reqs_in_sys.last_mod_ts)*reqs_in_sys.count as f64;
                reqs_in_sys.count += 1;
                reqs_in_sys.last_mod_ts = sim.time;
                if n_threads < sys.threadpool_size {
                    if let Some(rc_cpu) = idle_cpus.pop() {
                        events.push(process_request(rc_req, rc_cpu, sim.time, sys.quantum));
                    } else {
                        tpool.push_back(rc_req);
                    }
                    n_threads += 1;
                } else if rbuff.len() < sys.buffer_capacity {
                    rbuff.push_back(rc_req);
                } else {
                    sim.n_dropped += 1;
                    reqs_in_sys.count -= 1;
                    // The client cannot know the request was dropped right away.
                    // Therefore waits for a timeout, and then a retry think time,
                    // before issuing a new request.
                    let arrival_ts = sim.time + timeout_sampler.ind_sample(&mut rng) +
                                                sample_zero_lo(&retry_think_sampler, &mut rng);
                    let total_service = service_sampler.ind_sample(&mut rng);
                    let timeout = timeout_sampler.ind_sample(&mut rng);
                    let (arrival_e, timeout_e) = Event::new_arrival(arrival_ts, total_service, timeout);
                    events.push(arrival_e);
                    events.push(timeout_e);
                }
            },
            Departure(rc_cpu) => {
                //println!("T={} Departure {:?}", sim.time, rc_cpu.borrow());
                sim.wt_sum_reqs_in_sys += (sim.time - reqs_in_sys.last_mod_ts)*reqs_in_sys.count as f64;
                reqs_in_sys.count -= 1;
                reqs_in_sys.last_mod_ts = sim.time;
                {
                    let mut cpu = rc_cpu.borrow_mut();
                    let rc_req = match cpu.state {
                        CpuState::Busy(ref rc_req) => rc_req.clone(),
                        CpuState::Idle => panic!("Fatal: Cpu was Idle at a Departure!"),
                    };
                    if weak_count(&rc_req) > 0 { // Request was not timed out
                        let arrival_ts = sim.time + sample_zero_lo(&think_sampler, &mut rng);
                        let total_service = service_sampler.ind_sample(&mut rng);
                        let timeout = timeout_sampler.ind_sample(&mut rng);
                        let (arrival_e, timeout_e) = Event::new_arrival(arrival_ts, total_service, timeout);
                        events.push(arrival_e);
                        events.push(timeout_e);
                    } else {
                        reqs_in_sys.to_count -= 1;
                    }
                    cpu.total_busy_time += sim.time - cpu.quantum_start;
                    sim.sum_resp_time += sim.time - rc_req.borrow().arrival_time;
                    sim.n_processed += 1;
                }

                if let Some(req) = tpool.pop_front() {
                    events.push(process_request(req, rc_cpu, sim.time, sys.quantum));
                    if rbuff.len() > 0 {
                        tpool.push_back(rbuff.pop_front().unwrap());
                    } else {
                        n_threads -= 1;
                    }
                } else if let Some(req) = rbuff.pop_front() {
                    events.push(process_request(req, rc_cpu, sim.time, sys.quantum));
                } else {
                    n_threads -= 1;
                    rc_cpu.borrow_mut().state = CpuState::Idle;
                    idle_cpus.push(rc_cpu);
                }
            },
            QuantumOver(rc_cpu) => {
                //println!("T={} QuantumOver {:?}", sim.time, rc_cpu.borrow());
                {
                    let mut cpu = rc_cpu.borrow_mut();
                    let procd_time = sim.time - cpu.quantum_start;
                    cpu.total_busy_time += procd_time;
                    if let CpuState::Busy( ref rc_req ) = cpu.state {
                        rc_req.borrow_mut().remaining_service -= procd_time;
                        tpool.push_back(rc_req.clone());
                    } else {
                        panic!("Cpu should have been busy!");
                    }
                }
                events.push(process_request(tpool.pop_front().unwrap(), rc_cpu, sim.time, sys.quantum));
            },
            Timeout(weak_req) => match weak_req.upgrade() {
                Some(rc_req) => {
                    //println!("T={} Timedout! {:?}", sim.time, rc_req.borrow());
                    sim.n_timedout += 1;
                    reqs_in_sys.to_count += 1;
                    let arrival_ts = sim.time + sample_zero_lo(&retry_think_sampler, &mut rng);
                    let total_service = service_sampler.ind_sample(&mut rng);
                    let timeout = timeout_sampler.ind_sample(&mut rng);
                    let (arrival_e, timeout_e) = Event::new_arrival(arrival_ts, total_service, timeout);
                    events.push(arrival_e);
                    events.push(timeout_e);
                },
                None => {}
            },
        }
        iters += 1;
        if iters >= sys.max_iters {
            break;
        }
    }
    sim.total_cpu_time = cpus.into_iter().fold(0.0, |sum, cpu_rc| {
        let cpu = cpu_rc.borrow();
        sum + cpu.total_busy_time
    });
    sim.n_to_in_proc = reqs_in_sys.to_count;
    sim
}


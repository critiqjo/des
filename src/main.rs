#![feature(alloc)]
// suppress warning when using Weak, downgrade, weak_count

mod cpu;
mod event;
mod request;
mod simulation;

extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;

use std::io::{self, Read};
use std::borrow::Borrow;
use rustc_serialize::json;

fn main() {
    let mut stdin = io::stdin();
    let mut sys_json = String::new();
    let sys: simulation::SystemParams;
    sys = match stdin.read_to_string(&mut sys_json) {
        Ok(_) => json::decode(sys_json.borrow()).unwrap(),
        Err(e) => panic!("Fatal error: {:?}", e),
    };
    let sim = simulation::run(&sys);
    let avg_arrival_rate = sim.n_arrivals as f64 / sim.time;
    let tput = sim.n_processed as f64/sim.time;
    let n_processed_to = sim.n_timedout - sim.n_to_in_proc;
    let gput = (sim.n_processed - n_processed_to) as f64/sim.time;
    let avg_resp_time = sim.sum_resp_time/sim.n_processed as f64;
    let avg_cpu_util = sim.total_cpu_time / sim.time / sys.n_cpu as f64;
    let avg_reqs_in_sys = sim.wt_sum_reqs_in_sys / sim.total_cpu_time;
    let ffrac = (sim.n_dropped + n_processed_to) as f64/(sim.n_dropped + sim.n_processed) as f64;
    println!("
  Avg arrival rate = {}
  Avg throughput = {}
  Avg goodput = {}
  Avg response time = {}
  Avg CPU utilization = {}
  Avg number of requests in system = {}
  Fraction of failed requests = {}\n", avg_arrival_rate, tput, gput, avg_resp_time, avg_cpu_util, avg_reqs_in_sys, ffrac);
}

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
    let avg_resp_time = sim.sum_resp_time/sim.n_req_proc as f64;
    let avg_cpu_util = sim.total_cpu_time / sim.time / sys.n_cpu as f64;
    let avg_service_time = sim.total_cpu_time / sim.n_req_proc as f64;
    let tput = sim.n_req_proc as f64/sim.time;
    let utput = (sim.n_req_proc - sim.n_req_timeo) as f64/sim.time;
    let avg_reqs_in_sys = sim.wt_sum_reqs_in_sys / sim.total_cpu_time;
    let ffrac = (sim.n_req_drops + sim.n_req_timeo) as f64/(sim.n_req_proc + sim.n_req_drops) as f64;
    println!("
  Avg response time = {}
  Avg CPU utilization = {}
  Avg throughput = {}
  Avg useful throughput = {}
  Avg number of requests in system = {}
  Fraction of failed requests = {}
  Avg service time = {}\n", avg_resp_time, avg_cpu_util, tput, utput, avg_reqs_in_sys, ffrac, avg_service_time);
}

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

#[derive(RustcDecodable, RustcEncodable)]
struct SimResult {
    arrival_rate: f64,
    throughput: f64,
    goodput: f64,
    resp_time: f64,
    cpu_util: f64,
    reqs_in_sys: f64,
    failed_frac: f64,
}

fn main() {
    let mut stdin = io::stdin();
    let mut sys_json = String::new();
    let sys: simulation::SystemParams;
    sys = match stdin.read_to_string(&mut sys_json) {
        Ok(_) => json::decode(sys_json.borrow()).unwrap(),
        Err(e) => panic!("Fatal error: {:?}", e),
    };
    let sim = simulation::run(&sys);
    let n_processed_to = sim.n_timedout - sim.n_to_in_proc;
    let sim_res = SimResult {
        arrival_rate: sim.n_arrivals as f64 / sim.time,
        throughput: sim.n_processed as f64/sim.time,
        goodput: (sim.n_processed - n_processed_to) as f64/sim.time,
        resp_time: sim.sum_resp_time/sim.n_processed as f64,
        cpu_util: sim.total_cpu_time / sim.time / sys.n_cpu as f64,
        reqs_in_sys: sim.wt_sum_reqs_in_sys / sim.total_cpu_time,
        failed_frac: (sim.n_dropped + n_processed_to) as f64/(sim.n_dropped + sim.n_processed) as f64,
    };
    println!("{}", json::as_pretty_json(&sim_res));
}

#![feature(alloc)]
// suppress warning when using Weak, downgrade, weak_count

mod cpu;
mod event;
mod request;
mod simulation;

extern crate rand;
extern crate rustc_serialize;

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
    ctxx_busytime_frac: f64,
    reqs_in_sys: f64,
    dropped_frac: f64,
    drop_rate: f64,
    timedout_frac: f64,
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
    let total_cpu_time = sim.total_procd_time + sim.total_ctxx_time;
    let sim_res = SimResult {
        arrival_rate: sim.n_arrivals as f64 / sim.time,
        throughput: sim.n_processed as f64/sim.time,
        goodput: (sim.n_processed - n_processed_to) as f64/sim.time,
        resp_time: sim.sum_resp_time/sim.n_processed as f64,
        cpu_util: total_cpu_time / sim.time / sys.n_cpu as f64,
        ctxx_busytime_frac: sim.total_ctxx_time / total_cpu_time,
        reqs_in_sys: sim.wt_sum_reqs_in_sys / sim.total_procd_time,
        dropped_frac: sim.n_dropped as f64/(sim.n_dropped + sim.n_processed) as f64,
        drop_rate: sim.n_dropped as f64/sim.time,
        timedout_frac: n_processed_to as f64/(sim.n_dropped + sim.n_processed) as f64,
    };
    println!("{}", json::as_pretty_json(&sim_res));
}

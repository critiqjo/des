#![feature(alloc)]
// suppress warning when using Weak, downgrade, weak_count

mod cpu;
mod event;
mod request;
mod simulation;

extern crate rand;

fn main() {
    let sys = simulation::SystemParams {
        n_cpu: 4,
        n_users: 32,
        ease_in_time: 20.0,
        max_iters: 50000,
        buffer_capacity: 1000,
        threadpool_size: 40,
        quantum: 0.5,

        req_service_time_mean: 2.0,
        req_timeout_min: 10.0,
        req_timeout_max: 30.0,

        think_time_mean: 24.0,
        think_time_std_dev: 8.0,
        retry_think_time_mean: 2.0,
        retry_think_time_std_dev: 1.0,
    };
    let sim = simulation::run(&sys);
    let avg_resp_time = sim.sum_resp_time/sim.n_req_proc as f64;
    let avg_cpu_util = sim.total_cpu_time / sim.time / sys.n_cpu as f64;
    let avg_service_time = sim.total_cpu_time / sim.n_req_proc as f64;
    let tput = sim.n_req_proc as f64/sim.time;
    let utput = (sim.n_req_proc - sim.n_req_timeo) as f64/sim.time;
    let ffrac = (sim.n_req_drops + sim.n_req_timeo) as f64/(sim.n_req_proc + sim.n_req_drops) as f64;
    println!("
  Avg response time = {}
  Avg CPU utilization = {}
  Avg throughput = {}
  Avg useful throughput = {}
  Fraction of failed requests = {}
  Avg service time = {}\n", avg_resp_time, avg_cpu_util, tput, utput, ffrac, avg_service_time);
}

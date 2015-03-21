use rand::ThreadRng;
use rand::distributions::IndependentSample;

#[derive(Debug)]
pub struct Request {
    pub id: usize,
    pub arrival_time: f64,
    pub total_service: f64,
    pub remaining_service: f64,
}

static mut next_id:usize = 0;
impl Request {
    pub fn new<A: IndependentSample<f64>, S: IndependentSample<f64>>
        (now: f64, arrival_sampler: &A, service_sampler: &S, rng: &mut ThreadRng) -> Request {
        unsafe { next_id += 1; }
        let arrival = now + arrival_sampler.ind_sample(rng);
        let service = service_sampler.ind_sample(rng);
        Request { id: unsafe { next_id }, arrival_time: arrival, total_service: service, remaining_service: service }
    }
}


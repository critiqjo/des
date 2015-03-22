#[derive(Debug)]
pub struct Request {
    pub id: usize,
    pub arrival_time: f64,
    pub total_service: f64,
    pub remaining_service: f64,
}

static mut next_id:usize = 0;
impl Request {
    pub fn new(arrival_ts: f64, service_time: f64) -> Request {
        unsafe { next_id += 1; }
        Request { id: unsafe { next_id },
                  arrival_time: arrival_ts,
                  total_service: service_time,
                  remaining_service: service_time }
    }
}


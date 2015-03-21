#[derive(Debug)]
pub struct Request {
    pub id: usize,
    pub arrival_time: f64,
    pub total_service: f64,
    pub remaining_service: f64,
}

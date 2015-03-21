#[derive(Debug)]
pub struct Request {
    pub id: usize,
    pub arrival_time: usize,
    pub total_service: usize,
    pub remaining_service: usize,
}

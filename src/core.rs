use request::Request;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
#[derive(Debug)]
pub struct Core {
    pub status: CoreStatus,
    pub request: Option<Rc<RefCell<Request>>>,
    pub quantum_start: usize,
    pub total_busy_time: usize,
}


#[derive(Debug, Eq, PartialEq)]
enum CoreStatus {
    Idle, Busy
}

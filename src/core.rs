use std::rc::Rc;
use std::cell::RefCell;

use request::Request;

#[derive(Debug)]
pub struct Core {
    pub status: CoreStatus,
    pub request: Option<Rc<RefCell<Request>>>,
    pub quantum_start: usize,
    pub total_busy_time: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CoreStatus {
    Idle, Busy
}

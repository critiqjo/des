use std::rc::Rc;
use std::cell::RefCell;

use request::Request;

#[derive(Debug, Eq, PartialEq)]
pub enum CpuState {
    Idle, Busy
}

#[derive(Debug)]
pub struct Cpu {
    pub state: CpuState,
    pub request: Option<Rc<RefCell<Request>>>,
    pub quantum_start: usize,
    pub total_busy_time: usize,
}


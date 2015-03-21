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
    pub quantum_start: f64,
    pub total_busy_time: f64,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { state: CpuState::Idle, request: None, quantum_start: 0.0, total_busy_time: 0.0 }
    }
}


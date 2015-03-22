use std::rc::Rc;
use std::cell::RefCell;

use request::Request;

#[derive(Debug)]
pub enum CpuState {
    Idle, Busy(Rc<RefCell<Request>>)
}

#[derive(Debug)]
pub struct Cpu {
    pub state: CpuState,
    pub quantum_start: f64,
    pub total_busy_time: f64,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { state: CpuState::Idle, quantum_start: 0.0, total_busy_time: 0.0 }
    }
}


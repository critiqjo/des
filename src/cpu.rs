use std::rc::Rc;
use std::cell::RefCell;

use request::Request;

#[derive(Debug)]
pub enum CpuState {
    Idle,
    Busy(Rc<RefCell<Request>>, f64), // (req, quantum_start)
    CtxSwitching(Rc<RefCell<Request>>, Rc<RefCell<Request>>, f64), // (new, old, ctxx_start)
}

#[derive(Debug)]
pub struct Cpu {
    pub state: CpuState,
    pub total_procd_time: f64,
    pub total_ctxx_time: f64,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { state: CpuState::Idle, total_procd_time: 0.0, total_ctxx_time: 0.0 }
    }
}


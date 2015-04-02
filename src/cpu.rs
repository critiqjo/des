use std::rc::Rc;
use std::cell::RefCell;

use request::Request;

type RcRequest = Rc<RefCell<Request>>;
type NewRequest = RcRequest;
type OldRequest = RcRequest;
type QuantumStart = f64;
type CtxxStart = f64;

#[derive(Debug)]
pub enum CpuState {
    Idle,
    Busy(RcRequest, QuantumStart),
    CtxSwitching(NewRequest, OldRequest, CtxxStart),
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


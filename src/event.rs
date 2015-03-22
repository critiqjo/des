use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::cmp::Ordering;

use cpu::Cpu;
use request::Request;

#[derive(Debug)]
pub enum EventType {
    Arrival(Rc<RefCell<Request>>),
    Departure(Rc<RefCell<Cpu>>),
    QuantumOver(Rc<RefCell<Cpu>>),
    Timeout(Weak<RefCell<Request>>)
}
// EventType impl {{{
impl PartialEq for EventType {
    fn eq(&self, other: &EventType) -> bool {
        use self::EventType::*;
        match (self, other) {
            (&Arrival(_), &Arrival(_)) |
                (&Departure(_), &Departure(_)) |
                (&QuantumOver(_), &QuantumOver(_)) |
                (&Timeout(_), &Timeout(_)) => true,
                _ => false
        }
    }
}
impl Eq for EventType { }
impl PartialOrd for EventType {
    fn partial_cmp(&self, other: &EventType) -> Option<Ordering> {
        if self.eq(&other) {
            return Some(Ordering::Equal);
        }
        match (self, other) {
            (&EventType::Timeout(_), _) => Some(Ordering::Greater),
            (_, &EventType::Timeout(_)) => Some(Ordering::Less),
            _ => None,
        }
    }
}
// EventType }}}

#[derive(Debug)]
pub struct Event {
    pub _type: EventType,
    pub timestamp: f64,
}
// Event impl {{{
impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        // Consider incomparable 'EventType's as equals too
        self.timestamp == other.timestamp && !(self._type > other._type || self._type < other._type)
    }
}
impl Eq for Event { }
impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        // Notice that the we flip the ordering here to make it a min-heap
        match other.timestamp.partial_cmp(&self.timestamp).unwrap() {
            Ordering::Equal => self._type.partial_cmp(&other._type).unwrap(),
            ord => ord,
        }
    }
}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Event {
    pub fn new(_type: EventType, timestamp: f64) -> Event {
        return Event { _type: _type, timestamp: timestamp };
    }
    pub fn new_arrival(arrival_ts: f64, total_service: f64, timeout: f64) -> (Event, Event) {
        let req = Rc::new(RefCell::new(Request::new(arrival_ts, total_service)));

        let arrival_e = Event::new(EventType::Arrival(req.clone()), arrival_ts);
        let timeout_e = Event::new(EventType::Timeout(req.downgrade()), arrival_ts + timeout );
        (arrival_e, timeout_e)
    }
}
// Event }}}


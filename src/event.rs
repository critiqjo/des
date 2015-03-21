use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::cmp::Ordering;

use core::Core;
use request::Request;

#[derive(Debug)]
pub enum EventType {
    Arrival(Rc<RefCell<Request>>),
    Departure(Rc<RefCell<Core>>),
    QuantumOver(Rc<RefCell<Core>>),
    Timeout(Weak<RefCell<Request>>)
}
// EventType impl {{{
impl PartialEq for EventType {
    fn eq(&self, other: &EventType) -> bool {
        //        use EventType::*;
        match (self, other) {
            (&EventType::Arrival(_), &EventType::Arrival(_)) | (&EventType::Departure(_), &EventType::Departure(_)) |
                (&EventType::QuantumOver(_), &EventType::QuantumOver(_)) | (&EventType::Timeout(_), &EventType::Timeout(_))
                => true,
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
    pub timestamp: usize,
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
        match other.timestamp.cmp(&self.timestamp) {
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
// Event }}}


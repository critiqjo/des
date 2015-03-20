use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Debug, Formatter, Error};

#[derive(Debug)]
struct Request {
    id: usize,
    arrival_time: usize,
    total_service: usize,
    remaining_service: usize,
    timedout: bool,
}

// EventType enumerator {{{
#[derive(Debug, Eq, PartialEq)]
enum EventType {
    Arrival, Departure, QuantumOver, Timeout
}
impl PartialOrd for EventType {
    fn partial_cmp(&self, other: &EventType) -> Option<Ordering> {
        if self.eq(&other) {
             return Some(Ordering::Equal);
        }
        match (self, other) {
            (&EventType::Timeout, _) => Some(Ordering::Greater),
            (_, &EventType::Timeout) => Some(Ordering::Less),
            _ => None,
        }
    }
}
// EventType }}}

// Event structure {{{
#[derive(Debug)]
struct Event {
    _type: EventType,
    timestamp: usize,
    request: Option<RefCell<Request>>,
    //core: Option<RefCell<Core>>
}
impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.timestamp == other.timestamp && !(self._type.gt(&other._type) || self._type.lt(&other._type))
    }
}
impl Eq for Event { }
impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        if self.eq(&other) {
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

fn main() {
    let r = Request { id: 23, arrival_time: 434, total_service: 12, remaining_service: 12, timedout: false };
    println!("{:?}", &r);
    assert!(EventType::Timeout > EventType::Departure);
    assert!(EventType::Arrival < EventType::Timeout);
}

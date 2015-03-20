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
    use EventType::{Arrival, Departure, Timeout, QuantumOver};
    let mut events = BinaryHeap::new();
    let e = Event { _type: Arrival, timestamp: 4, request: None };
    events.push(e);
    let e = Event { _type: Departure, timestamp: 8, request: None };
    events.push(e);
    let e = Event { _type: Timeout, timestamp: 8, request: None };
    events.push(e);
    let c = events.pop().unwrap();
    println!("{:?}", &c);
    let c = events.pop().unwrap();
    println!("{:?}", &c);
    let c = events.pop().unwrap();
    println!("{:?}", &c);
    assert_eq!(events.pop(), None);
}

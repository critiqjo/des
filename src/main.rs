#![feature(alloc)]
// suppress warning when using Weak, downgrade, strong_count

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BinaryHeap;

extern crate rand;
use rand::distributions::IndependentSample;
use rand::distributions::exponential::Exp;

mod core;
mod event;
mod request;
use core::Core;
use event::{Event, EventType};
use request::Request;

fn main() {
    let mut events = BinaryHeap::new();
    let mut rng = rand::thread_rng();
    let exp = Exp::new(1.0/8.0);

    let mut v = exp.ind_sample(&mut rng);
    println!("rand: {}", v);

    let req = Rc::new(RefCell::new(Request { id: 89, arrival_time: v as usize, total_service: 4, remaining_service: 4 }));
    let mut e = Event { _type: EventType::Arrival(req.clone()), timestamp: req.borrow().arrival_time };
    events.push(e);

    v += exp.ind_sample(&mut rng);
    e = Event { _type: EventType::Timeout(req.clone().downgrade()), timestamp: v as usize };
    events.push(e);

    let mut e = events.pop().unwrap();
    println!("{:?}", &e);
    e = events.pop().unwrap();
    println!("{:?}", &e);
    assert_eq!(events.pop(), None);
}


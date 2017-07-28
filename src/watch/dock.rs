use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::thread::{self, Thread, ThreadId};
use std::time::Instant;

use actor::{self, Actor};

// TODO: hide pub fields

pub struct Request<T> {
    pub actor: Arc<Actor>,
    pub data: UnsafeCell<Option<T>>,
}

impl<T> Request<T> {
    pub fn new(data: Option<T>) -> Self {
        Request {
            actor: actor::current(),
            data: UnsafeCell::new(data),
        }
    }

    // TODO put(value: T)
    // TODO take() -> T
}

pub struct Blocked<T> {
    actor: Arc<Actor>,
    data: Option<*const UnsafeCell<Option<T>>>,
}

pub struct Deque<T>(VecDeque<Blocked<T>>);

impl<T> Deque<T> {
    pub fn new() -> Self {
        Deque(VecDeque::new())
    }

    pub fn pop(&mut self) -> Option<Blocked<T>> {
        self.0.pop_front()
    }

    pub fn register(&mut self, data: Option<*const UnsafeCell<Option<T>>>) {
        self.0.push_back(Blocked {
            actor: actor::current(),
            data,
        });
    }

    pub fn unregister(&mut self) {
        // TODO: data argument
        let id = thread::current().id();
        self.0.retain(|s| s.actor.thread_id() != id); // TODO: use find, enumerate, remove with data argument?
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn notify_all(&mut self) {
        for t in self.0.drain(..) {
            if t.actor.select(1) {
                t.actor.unpark();
            }
        }
    }
}

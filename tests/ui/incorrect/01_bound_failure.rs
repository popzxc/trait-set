use trait_set::trait_set;
use std::cell::RefCell;

trait_set! {
    pub trait ThreadSafe = Send + Sync;
}

fn test<T: ThreadSafe>(_t: T) {}

fn main() {
    test(RefCell::new(10u8));
}

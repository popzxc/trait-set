//! Checks that type parameter defaults stay on the alias declaration but are
//! omitted from the generated blanket implementation.

use trait_set::trait_set;

trait Marker<T> {}

impl Marker<u8> for u8 {}

trait_set! {
    pub(crate) trait Defaulted<T = u8> = Marker<T>;
}

fn test<T: Defaulted>(_value: T) {}

fn main() {
    test(10u8);
}

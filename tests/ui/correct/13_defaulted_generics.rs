//! Checks that type parameter defaults stay on the alias declaration but are
//! omitted from the generated blanket implementation.

use trait_set::trait_set;

trait Marker<T> {}

impl Marker<u8> for u8 {}

trait ConstMarker<const N: usize> {}

impl ConstMarker<4> for [u8; 4] {}

trait_set! {
    pub(crate) trait Defaulted<T = u8> = Marker<T>;
    pub(crate) trait ConstDefaulted<const N: usize = 4> = ConstMarker<N>;
}

fn test<T: Defaulted>(_value: T) {}
fn test_const<T: ConstDefaulted>(_value: T) {}

fn main() {
    test(10u8);
    test_const([0; 4]);
}

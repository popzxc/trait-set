//! Checks that const generic declarations become const generic arguments in
//! the generated blanket implementation.

use trait_set::trait_set;

trait ConstMarker<const N: usize> {}

impl<const N: usize> ConstMarker<N> for [u8; N] {}

trait_set! {
    pub(crate) trait Fixed<const N: usize> = ConstMarker<N>;
}

fn test<const N: usize, T: Fixed<N>>(_value: T) {}

fn main() {
    test::<4, _>([0; 4]);
}

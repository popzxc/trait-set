//! Checks that `_INNER`, the blanket implementation's former type parameter
//! name, can now be used as an alias parameter.

#![allow(non_camel_case_types)]

use trait_set::trait_set;

trait Marker<T> {}

impl Marker<u8> for () {}

trait_set! {
    pub(crate) trait Alias<_INNER> = Marker<_INNER>;
}

fn test<T: Alias<u8>>(_value: T) {}

fn main() {
    test(());
}

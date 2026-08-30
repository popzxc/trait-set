//! Checks that lifetime bounds remain on declarations but are omitted from
//! the generated trait arguments.

use trait_set::trait_set;

trait Borrowed<'a> {}

impl<'a> Borrowed<'a> for &'a str {}

trait_set! {
    pub(crate) trait Outlives<'a: 'b, 'b> = Borrowed<'a> + 'b;
}

fn test<T: Outlives<'static, 'static>>(_value: T) {}

fn main() {
    test("a static string");
}

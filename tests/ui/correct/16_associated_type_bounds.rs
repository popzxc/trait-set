//! Regression test for issue #7: associated types can have trait bounds in
//! addition to equality constraints.

use trait_set::trait_set;

trait HasParent {}

struct Parent;

impl HasParent for Parent {}

trait Resource {
    type DynamicType;
}

trait HasInnerSpec {
    type InnerSpec;
}

struct Representation;

impl Resource for Representation {
    type DynamicType = ();
}

impl HasInnerSpec for Representation {
    type InnerSpec = Parent;
}

trait_set! {
    pub(crate) trait RepresentationBounds = Resource<DynamicType = ()>
        + HasInnerSpec<InnerSpec: HasParent>;
}

fn test<T: RepresentationBounds>(_value: T) {}

fn main() {
    test(Representation);
}

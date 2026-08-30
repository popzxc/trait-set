//! Checks that configuration attributes guard both generated items.

#![allow(deprecated)]

use trait_set::trait_set;

trait_set! {
    // Neither missing bound should be resolved: both aliases and their
    // blanket implementations are disabled by their attributes.
    #[cfg(any())]
    pub(crate) trait DirectlyDisabled = MissingTrait;

    #[cfg_attr(all(), cfg(any()))]
    pub(crate) trait IndirectlyDisabled<T> = MissingGenericTrait<T>;

    // Non-configuration metadata remains on the public trait and is not
    // copied onto the blanket implementation, where Rust would reject it.
    #[cfg_attr(all(), deprecated(note = "kept on the alias trait"))]
    pub(crate) trait DeprecatedAlias = Send;
}

fn test<T: DeprecatedAlias>(_value: T) {}

fn main() {
    test(());
}

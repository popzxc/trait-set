//! This crate provide support for [trait aliases][alias]: a feature
//! that is already supported by Rust compiler, but is [not stable][tracking_issue]
//! yet.
//!
//! The idea is simple: combine group of traits under a single name. The simplest
//! example will be:
//!
//! ```rust
//! use trait_set::trait_set;
//!
//! trait_set! {
//!     pub trait ThreadSafe = Send + Sync;
//! }
//! ```
//!
//! Macro [`trait_set`] displayed here is the main entity of the crate:
//! it allows declaring multiple trait aliases, each of them is represented
//! as
//!
//! ```text
//! [visibility] trait [AliasName][<generics>] = [Element1] + [Element2] + ... + [ElementN];
//! ```
//!
//! For more details, see the [`trait_set`] macro documentation.
//!
//! [alias]: https://doc.rust-lang.org/unstable-book/language-features/trait-alias.html
//! [tracking_issue]: https://github.com/rust-lang/rust/issues/41517
//! [`trait_set`]: macro.trait_set.html

extern crate proc_macro;

use std::iter::FromIterator;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Error, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Generics, Ident, Result, Token, TypeTraitObject, Visibility,
};

/// Represents one trait alias.
struct TraitSet {
    visibility: Visibility,
    _trait_token: Token![trait],
    alias_name: Ident,
    generics: Generics,
    _eq_token: Token![=],
    traits: TypeTraitObject,
}

impl TraitSet {
    /// Renders trait alias into a new trait with bounds set.
    fn render(self) -> TokenStream2 {
        // Generic and non-generic implementation have slightly different
        // syntax, so it's simpler to process them individually rather than
        // try to generalize implementation.
        if self.generics.params.is_empty() {
            self.render_non_generic()
        } else {
            self.render_generic()
        }
    }

    /// Renders the trait alias without generic parameters.
    fn render_non_generic(self) -> TokenStream2 {
        let visibility = self.visibility;
        let alias_name = self.alias_name;
        let bounds = self.traits.bounds;
        quote! {
            #visibility trait #alias_name: #bounds {}

            impl<_INNER> #alias_name for _INNER where _INNER: #bounds {}
        }
    }

    /// Renders the trait alias with generic parameters.
    fn render_generic(self) -> TokenStream2 {
        let visibility = self.visibility;
        let alias_name = self.alias_name;
        let bounds = self.traits.bounds;
        let generics = self.generics.params;
        // Note that it's important for `_INNER` to go *after* user-defined
        // generics, because generics can contain lifetimes, and lifetimes
        // should always go first.
        quote! {
            #visibility trait #alias_name<#generics>: #bounds {}

            impl<#generics, _INNER> #alias_name<#generics> for _INNER where _INNER: #bounds {}
        }
    }
}

impl Parse for TraitSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let result = TraitSet {
            visibility: input.parse()?,
            _trait_token: input.parse()?,
            alias_name: input.parse()?,
            generics: input.parse()?,
            _eq_token: input.parse()?,
            traits: input.parse()?,
        };

        if let Some(where_clause) = result.generics.where_clause {
            return Err(Error::new(
                where_clause.span(),
                "Where clause is not allowed for trait alias",
            ));
        }
        Ok(result)
    }
}

/// Represents a sequence of trait aliases delimited by semicolon.
struct ManyTraitSet {
    entries: Punctuated<TraitSet, Token![;]>,
}

impl Parse for ManyTraitSet {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ManyTraitSet {
            entries: input.parse_terminated(TraitSet::parse)?,
        })
    }
}

impl ManyTraitSet {
    fn render(self) -> TokenStream2 {
        TokenStream2::from_iter(self.entries.into_iter().map(|entry| entry.render()))
    }
}

/// Creates an alias for set of traits.
///
/// To demonstrate the idea, see the examples:
///
/// ```rust
/// use trait_set::trait_set;
///
/// trait_set! {
///     pub trait ThreadSafe = Send + Sync;
///     pub trait ThreadSafeIterator<T> = ThreadSafe + Iterator<Item = T>;
///     pub trait ThreadSafeBytesIterator = ThreadSafeIterator<u8>;
///     pub trait StaticDebug = 'static + std::fmt::Debug;
/// }
///```
///
/// This macro also supports [higher-rank trait bound][hrtb]:
///
/// ```rust
/// # pub trait Serializer {
/// #     type Ok;
/// #     type Error;
/// #
/// #     fn ok_value() -> Self::Ok;
/// # }
/// # pub trait Deserializer<'de> {
/// #     type Error;
/// # }
/// #
/// # pub trait Serialize {
/// #     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
/// #     where
/// #         S: Serializer;
/// # }
/// #
/// # pub trait Deserialize<'de>: Sized {
/// #     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
/// #     where
/// #         D: Deserializer<'de>;
/// # }
/// #
/// # impl Serializer for u8 {
/// #     type Ok = ();
/// #     type Error = ();
/// #
/// #     fn ok_value() -> Self::Ok {
/// #         ()
/// #     }
/// # }
/// #
/// # impl<'de> Deserializer<'de> for u8 {
/// #     type Error = ();
/// # }
/// #
/// # impl Serialize for u8 {
/// #     fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
/// #     where
/// #         S: Serializer
/// #     {
/// #         Ok(S::ok_value())
/// #     }
/// # }
/// #
/// # impl<'de> Deserialize<'de> for u8 {
/// #     fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
/// #     where
/// #         D: Deserializer<'de>
/// #         {
/// #             Ok(0u8)
/// #         }
/// # }
/// use trait_set::trait_set;
///
/// trait_set!{
///     pub trait Serde = Serialize + for<'de> Deserialize<'de>;
///     // Note that you can also use lifetimes as a generic parameter.
///     pub trait SerdeLifetimeTemplate<'de> = Serialize + Deserialize<'de>;
/// }
/// ```
///
/// [hrtb]: https://doc.rust-lang.org/nomicon/hrtb.html
#[proc_macro]
pub fn trait_set(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ManyTraitSet);
    input.render().into()
}

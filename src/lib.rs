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
    Attribute, Generics, Ident, Meta, Result, Token, TypeTraitObject, Visibility,
};

/// Represents one trait alias.
struct TraitSet {
    attributes: Vec<Attribute>,
    implementation_attributes: Vec<Attribute>,
    visibility: Visibility,
    _trait_token: Token![trait],
    alias_name: Ident,
    generics: Generics,
    _eq_token: Token![=],
    traits: TypeTraitObject,
}

impl TraitSet {
    /// Selects configuration attributes that must also guard the generated
    /// implementation. Other attributes describe the user-facing trait and
    /// may not be valid on a trait implementation.
    fn implementation_attributes(attributes: &[Attribute]) -> Result<Vec<Attribute>> {
        attributes
            .iter()
            .map(|attribute| Self::configuration_meta(&attribute.meta))
            .filter_map(Result::transpose)
            .map(|meta| meta.map(|meta| syn::parse_quote!(#[#meta])))
            .collect()
    }

    /// Retains only the item-existence part of an attribute. In particular,
    /// `cfg_attr` can contain attributes such as `deprecated` that belong on
    /// the trait but would be rejected on its blanket implementation.
    fn configuration_meta(meta: &Meta) -> Result<Option<Meta>> {
        if meta.path().is_ident("cfg") {
            return Ok(Some(meta.clone()));
        }
        if !meta.path().is_ident("cfg_attr") {
            return Ok(None);
        }

        let Meta::List(list) = meta else {
            return Err(Error::new(meta.span(), "expected `cfg_attr(...)`"));
        };
        let arguments = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        let mut arguments = arguments.iter();
        let Some(predicate) = arguments.next() else {
            return Err(Error::new(meta.span(), "missing `cfg_attr` predicate"));
        };
        let configuration_attributes = arguments
            .map(Self::configuration_meta)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(if configuration_attributes.is_empty() {
            None
        } else {
            Some(syn::parse_quote! {
                cfg_attr(#predicate, #(#configuration_attributes),*)
            })
        })
    }

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
        let attributes = self.attributes;
        let implementation_attributes = self.implementation_attributes;
        let visibility = self.visibility;
        let alias_name = self.alias_name;
        let bounds = self.traits.bounds;
        quote! {
            #(#attributes)*
            #visibility trait #alias_name: #bounds {}

            #(#implementation_attributes)*
            impl<_INNER> #alias_name for _INNER where _INNER: #bounds {}
        }
    }

    /// Renders the trait alias with generic parameters.
    fn render_generic(self) -> TokenStream2 {
        let attributes = self.attributes;
        let implementation_attributes = self.implementation_attributes;
        let visibility = self.visibility;
        let alias_name = self.alias_name;
        let bounds = self.traits.bounds;
        let generics = self.generics;

        // Syn owns the distinction between parameter declarations (`T: Send`,
        // `const N: usize`) and their use as arguments (`T`, `N`). Besides
        // preserving every supported generic form, `split_for_impl` removes
        // defaults where Rust forbids them in an impl declaration.
        let mut implementation_generics = generics.clone();
        implementation_generics
            .params
            .push(syn::parse_quote!(__TRAIT_SET_INNER));
        let (implementation_generics, _, _) = implementation_generics.split_for_impl();
        let (_, alias_generics, _) = generics.split_for_impl();

        quote! {
            #(#attributes)*
            #visibility trait #alias_name #generics: #bounds {}

            #(#implementation_attributes)*
            impl #implementation_generics #alias_name #alias_generics for __TRAIT_SET_INNER
            where
                __TRAIT_SET_INNER: #bounds
            {}
        }
    }
}

impl Parse for TraitSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let implementation_attributes = Self::implementation_attributes(&attributes)?;
        let result = TraitSet {
            attributes,
            implementation_attributes,
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
            entries: input.parse_terminated(TraitSet::parse, Token![;])?,
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
///     /// Doc-comments are also supported btw.
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

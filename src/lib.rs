extern crate proc_macro;

use proc_macro::{TokenStream};
use syn::{Generics, Ident, Result, Token, TypeTraitObject, Visibility, parse_macro_input, spanned::Spanned};
use syn::parse::{Parse, ParseStream, Error};
use syn::punctuated::Punctuated;
use quote::{quote};
use proc_macro2::TokenStream as TokenStream2;

struct TraitSet {
    visibility: Visibility,
    _trait_token: Token![trait],
    alias_name: Ident,
    generics: Generics,
    _eq_token: Token![=],
    traits: TypeTraitObject
}

impl TraitSet {
    fn render(self) -> TokenStream2 {
        if self.generics.params.is_empty() {
            self.render_non_generic()
        } else {
            self.render_generic()
        }
    }

    fn render_non_generic(self) -> TokenStream2 {
        let visibility = self.visibility;
        let alias_name = self.alias_name;
        let bounds = self.traits.bounds;
        quote! {
            #visibility trait #alias_name: #bounds {}

            impl<_INNER> #alias_name for _INNER where _INNER: #bounds {}
        }
    }

    fn render_generic(self) -> TokenStream2 {
        let visibility = self.visibility;
        let alias_name = self.alias_name;
        let bounds = self.traits.bounds;
        let generics = self.generics.params;
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
            return Err(Error::new(where_clause.span(), "Where clause is not allowed for trait alias"));
        }
        Ok(result)
    }
}

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
        let entries = self.entries.into_iter().map(|entry| entry.render());

        quote! {
            #(#entries)*
        }
    }
}

#[proc_macro]
pub fn trait_set(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ManyTraitSet);
    input.render().into()
}

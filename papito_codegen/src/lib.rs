#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate heck;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Item, Ident, Type, TypePath, PathSegment, PathArguments};
use syn::spanned::Spanned;
use syn::punctuated::Pair;
use heck::SnakeCase;
use proc_macro2::Span;

#[proc_macro_attribute]
pub fn component(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let state: Item = syn::parse(input).expect("Expected it to be an Item");
    let component = match state {
        Item::Struct(ref struct_item) => {
            let state_ident = &struct_item.ident;
            let comp_ident = Ident::new(&format!("{}Component", struct_item.ident), Span::call_site());
            let state_fields = &struct_item.fields;
            let vis = &struct_item.vis;
            quote_spanned! { Span::call_site() =>
                #vis struct #comp_ident {
                    inner: ::std::rc::Rc<::std::cell::RefCell<#state_ident>>
                }
            }
        }
        Item::Fn(_) => {
            quote! {}
        }
        _ => {
            state.span().unstable()
                .error("The attribute is only allowed for fns and structs")
                .emit();
            quote! {}
        }
    };
    let expanded = quote! {
        #state

        #component
    };
    expanded.into()
}
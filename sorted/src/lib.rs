mod utils;
mod visitor;

use proc_macro::TokenStream;

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Item, ItemFn};
use syn::visit_mut::VisitMut;
use crate::utils::find_unsorted_element;
use crate::visitor::CheckSortedMatch;

const ERROR_MESSAGE: &str = "expected enum or match expression";


#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_item: Item = parse_macro_input!(input);

    match _sorted(&input_item) {
        // Hand the output tokens back to the compiler
        Ok(_) => input_item.to_token_stream().into(),
        Err(error) => {
            let err: TokenStream2 = error.to_compile_error();
            let result: TokenStream2 = quote! { #input_item #err };
            result.into()
        },
    }
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_item: ItemFn = parse_macro_input!(input);

    match _check(input_item) {
        // Hand the output tokens back to the compiler
        Ok(result) => result.into(),
        Err((result, error)) => {
            let err: TokenStream2 = error.to_compile_error();
            let result: TokenStream2 = quote! { #result #err };
            result.into()
        },
    }
}


fn _sorted(input_item: &Item) -> Result<(), syn::Error> {
    // Only work on enums
    if let Item::Enum(input_enum) = input_item.clone() {

        let variants: Vec<&Ident> = input_enum.variants
            .iter()
            .map(|x| &x.ident)
            .collect();

        // Return with the full provided input if the list is sorted
        if variants.is_sorted() {
            return Ok(())
        }

        // Otherwise, build a correct error
        let (unsorted_index, should_be_index): (usize, usize) = find_unsorted_element(&variants);

        let unsorted_element: &Ident = &variants[unsorted_index];
        let should_be_elmt: &Ident = &variants[should_be_index];
        let error_message: String = format!("{} should sort before {}", unsorted_element, should_be_elmt);

        Err(syn::Error::new(variants[unsorted_index].span(), error_message))

    } else {
        // Raise a compile error if applied on something that is not an enum
        Err(syn::Error::new(Span::call_site(), ERROR_MESSAGE))
    }
}

fn _check(mut input_item_fn: ItemFn) -> Result<TokenStream2, (TokenStream2, syn::Error)> {
    let mut visitor: CheckSortedMatch = CheckSortedMatch::new();
    visitor.visit_item_fn_mut(&mut input_item_fn);

    let result: TokenStream2 = quote!{ #input_item_fn };

    if visitor.errors.is_empty() {
        Ok(result)
    } else {
        Err((result, visitor.errors.first().unwrap().clone()))
    }
}

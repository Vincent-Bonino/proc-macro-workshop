use quote::{quote, ToTokens};
use proc_macro::TokenStream;

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{parse_macro_input, Item};

const ERROR_MESSAGE: &str = "expected enum or match expression";


#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let _ = args;
    let input_item: Item = parse_macro_input!(input);

    match _sorted(None, &input_item) {
        // Hand the output tokens back to the compiler
        Ok(_) => input_item.to_token_stream().into(),
        Err(error) => {
            let err: TokenStream2 = error.to_compile_error();
            let result: TokenStream2 = quote! { #input_item #err };
            result.into()
        },
    }
}


fn _sorted(_args: Option<TokenStream>, input_item: &Item) -> Result<(), syn::Error> {
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


fn find_unsorted_element<T: PartialOrd>(array: &[T]) -> (usize, usize) {
    let length: usize = array.len();

    let mut unsorted_elmt_index: usize = 0;  // First unsorted element of the array
    let mut target_elmt_index: usize = 0;  // Position where the unsorted element should be

    // Find which element is not at the right place
    for i in 1..length {
        if array[i-1] > array[i] {
            unsorted_elmt_index = i;
            break;
        }
    }

    let unsorted_elmt: &T = &array[unsorted_elmt_index];

    // Determine where it should be
    for j in 0..length {
        if &array[j] > unsorted_elmt {
            target_elmt_index = j;
            break;
        }
    }

    (unsorted_elmt_index, target_elmt_index)
}

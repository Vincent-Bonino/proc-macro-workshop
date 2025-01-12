mod utils;


use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type, Visibility};
use syn::{Data, DataStruct, Fields, FieldsNamed};
use crate::utils::is_type_optional;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_derive: DeriveInput = parse_macro_input!(input);

    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input_derive.data
    {
        // ####################
        // #  Base structure  #
        // ####################
        let vis: Visibility = input_derive.vis;
        let name: Ident = input_derive.ident;

        // Fields of the base structure, identifier and type tuple
        let fields_it = named
            .iter()
            .map(|f| (f.ident.as_ref().expect("Unnamed field"), &f.ty));
        // Fields identifiers
        let fields_ident = fields_it
            .clone()
            .map(|(i, _t)| i);

        // #############
        // #  Builder  #
        // #############
        let builder_name: Ident = Ident::new(&format!("{}Builder", name), Span::call_site());

        // Support of optional fields is done with an extra Option wrapper
        // Required fields: None = not set // Some(_) = set
        // Optional fields: Some(None) = not set // Some(Some(_)) = set
        let builder_fields = fields_it
            .clone()
            .map(|(i, t)| builder_field(i, t));
        let builder_setters = fields_it
            .clone()
            .map(|(i, t)| builder_setter(i, t));
        let builder_defaults = fields_it
            .clone()
            .map(|(i, t)| builder_default(i, t));

        // Build the output, possibly using quasi-quotation

        let result = quote! {

            // Builder
            #[derive(Debug)]
            #vis struct #builder_name {
                #( #builder_fields ),*
            }

            impl #builder_name {
                #( #builder_setters )*

                pub fn build(&mut self) ->
                    ::core::result::Result<Command, std::boxed::Box<dyn ::std::error::Error>> {

                    Ok(
                        #name {
                            #(
                                #fields_ident : self.#fields_ident
                                    .take()
                                    .ok_or_else(||
                                        format!("Field {} must be set!", stringify!(#fields_ident))
                                    )?
                            ),*
                        }
                    )
                }
            }

            // Struct
            impl #name {
                pub fn builder() -> #builder_name {
                    #builder_name {
                        #( #builder_defaults ),*
                    }
                }
            }
        };

        // Hand the output tokens back to the compiler
        TokenStream::from(result)
    } else {
        unimplemented!("Named structures only")
    }
}


/// Build the TokenStream of a builder's field.
///
/// Support optional fields.
fn builder_field(ident: &Ident, ty: &Type) -> TokenStream2 {
    // match is_type_optional(ty) {
    //     // ty = Option<inty>
    //     Some(inty) => quote! { #ident : ::core::option::Option<::core::option::Option<#inty>> },
    //     None => quote! { #ident : ::core::option::Option<#ty> },
    // }
    quote! { #ident : ::core::option::Option<#ty> }
}

/// Build the TokenStream of a builder's setter.
///
/// Support optional fields.
fn builder_setter(ident: &Ident, ty: &Type) -> TokenStream2 {
    match is_type_optional(ty) {
        // ty = Option<inty>
        Some(inty) =>
            quote! {
                fn #ident (&mut self, value: #inty) -> &mut Self {
                    self.#ident = ::core::option::Option::Some(::core::option::Option::Some(value));
                    self
                }
            },
        None => quote! {
            fn #ident (&mut self, value: #ty) -> &mut Self {
                self.#ident = ::core::option::Option::Some(value);
                self
            }
        },
    }
}

/// Build the TokenStream of a builder's field default value.
///
/// Support optional fields.
fn builder_default(ident: &Ident, ty: &Type) -> TokenStream2 {
    match is_type_optional(ty) {
        // ty = Option<inty>
        Some(_) => quote! { #ident : ::core::option::Option::Some(::core::option::Option::None) },
        None => quote! { #ident : ::core::option::Option::None },
    }
}

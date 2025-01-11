use proc_macro::TokenStream;

use quote::quote;
use syn::__private::Span;
use syn::{parse_macro_input, DeriveInput, Ident, Visibility};
use syn::{Data, DataStruct, Fields, FieldsNamed};

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
        let _vis: Visibility = input_derive.vis;
        let name: Ident = input_derive.ident;

        let fields_it = named
            .iter()
            .map(|f| (f.ident.as_ref().expect("Unnamed field"), &f.ty));
        let fields_ident = fields_it
            .clone()
            .map(|(i, _t)| i);

        // #############
        // #  Builder  #
        // #############
        let builder_name: Ident = Ident::new(&format!("{}Builder", name), Span::call_site());

        let builder_fields = fields_it
            .clone()
            .map(|(i, t)| quote! { #i : ::core::option::Option<#t> });
        let builder_setters = fields_it.clone().map(|(i, t)| {
            quote! {
                fn #i (&mut self, value: #t) -> &mut Self {
                    self.#i = ::core::option::Option::Some(value);
                    self
                }
            }
        });

        // Build the output, possibly using quasi-quotation

        let result = quote! {

            // Builder
            struct #builder_name {
                #( #builder_fields ),*
            }

            impl #builder_name {
                #( #builder_setters )*

                pub fn build(&mut self) ->
                    ::core::result::Result<Command, std::boxed::Box<dyn ::std::error::Error>> {

                    Ok(
                        #name {
                            #(
                                #fields_ident : self.#fields_ident.as_ref().ok_or("None field")?.clone()
                            ),*
                        }
                    )
                }
            }

            // Struct
            impl #name {
                pub fn builder() -> #builder_name {
                    #builder_name {
                        executable: None,
                        args: None,
                        env: None,
                        current_dir: None,
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

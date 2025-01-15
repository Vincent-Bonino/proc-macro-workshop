use proc_macro2::{TokenStream};
use quote::ToTokens;
use syn::{visit_mut, Error, ExprMatch, Pat};
use syn::visit_mut::VisitMut;

use crate::utils::{find_unsorted_element, has_sorted_attribute};

pub(crate) struct CheckSortedMatch{
    pub(crate) errors: Vec<Error>,
}

impl CheckSortedMatch {
    pub fn new() -> Self {
        CheckSortedMatch {
            errors: Vec::new(),
        }
    }
}

impl VisitMut for CheckSortedMatch {
    fn visit_expr_match_mut(&mut self, i: &mut ExprMatch) {
        // Determine if the match expression has an attribute
        let sorted_attribute_idx_opt: Option<usize> = has_sorted_attribute(i);

        if let Some(sorted_attribute_index) = sorted_attribute_idx_opt {

            // For each arm, build a tuple:
            //  - String; use to determine if the match is sorted
            //  - TokenStream; token stream of the path, since path.span() does not work as expected
            let pattern_idents: Vec<(String, TokenStream)> = i.arms
                .iter()
                .filter_map(|x| {
                    match &x.pat {
                        Pat::Ident(ident) => {
                            Some((format!("{}", ident.ident), ident.to_token_stream()))
                        },
                        Pat::Path(path) => {
                            let parts: Vec<String> = path.path.segments
                                .iter()
                                .map(|x| format!("{}", x.ident)).collect();
                            Some((parts.join("::"), path.path.to_token_stream()))
                        },
                        Pat::Struct(struc) => {
                            let parts: Vec<String> = struc.path.segments
                                .iter()
                                .map(|x| format!("{}", x.ident)).collect();
                            Some((parts.join("::"), struc.path.to_token_stream()))
                        },
                        Pat::TupleStruct(tuple_struct) => {
                            let parts: Vec<String> = tuple_struct.path.segments
                                .iter()
                                .map(|x| { format!("{}", x.ident) }).collect();
                            Some((parts.join("::"), tuple_struct.path.to_token_stream()))
                        },
                        Pat::Wild(wild) => {
                            Some((String::from("_"), wild.underscore_token.to_token_stream()))
                        }
                        _ => {
                            self.errors.push(
                                syn::Error::new_spanned(x.pat.to_token_stream(), "unsupported by #[sorted]")
                            );
                            None
                        },
                    }
                }).collect();

            let ident_str_vec: Vec<&String> = pattern_idents.iter().map(|(i, _ts)| i).collect();
            let tokenstream_vec: Vec<&TokenStream> = pattern_idents.iter().map(|(_i, ts)| ts).collect();

            // Determine if the expression is sorted
            if !ident_str_vec.is_sorted() {
                let (unsorted_index, should_be_index): (usize, usize) = find_unsorted_element(&ident_str_vec);

                let unsorted_element: &str = &ident_str_vec[unsorted_index];
                let should_be_elmt: &str = &ident_str_vec[should_be_index];
                let error_message: String = format!("{} should sort before {}", unsorted_element, should_be_elmt);

                self.errors.push(syn::Error::new_spanned(tokenstream_vec[unsorted_index], error_message));
            }

            // Remove the attribute
            i.attrs.remove(sorted_attribute_index);
        }

        // Continue the visit
        visit_mut::visit_expr_match_mut(self, i);
    }
}

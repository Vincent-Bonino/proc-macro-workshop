use proc_macro2::Ident;
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

        // If there is no attribute, continue the visit
        if let Some(sorted_attribute_index) = sorted_attribute_idx_opt {
            let pattern_idents: Vec<&Ident> = i.arms
                .iter()
                .filter_map(|x| {
                    if let Pat::TupleStruct(tuple_struct) = &x.pat {
                        if let Some(path_segment) = tuple_struct.path.segments.first() {
                            Some(&path_segment.ident)
                        } else {
                            None
                        }
                    } else {
                        // Unsupported, filtered out
                        None
                    }
                }).collect();

            // Determine if the expression is sorted
            if !pattern_idents.is_sorted() {
                let (unsorted_index, should_be_index): (usize, usize) = find_unsorted_element(&pattern_idents);

                let unsorted_element: &Ident = &pattern_idents[unsorted_index];
                let should_be_elmt: &Ident = &pattern_idents[should_be_index];
                let error_message: String = format!("{} should sort before {}", unsorted_element, should_be_elmt);

                self.errors.push(syn::Error::new(pattern_idents[unsorted_index].span(), error_message));
            }

            // Remove the attribute
            i.attrs.remove(sorted_attribute_index);
        }

        // Continue the visit
        visit_mut::visit_expr_match_mut(self, i);
    }
}

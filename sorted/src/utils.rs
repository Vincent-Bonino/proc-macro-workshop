use syn::{ExprMatch, Meta};

pub(crate) fn find_unsorted_element<T: PartialOrd>(array: &[T]) -> (usize, usize) {
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

/// Return Some(idx) with idx being the index of the sorted attribute.
/// Return None if there is not sorted attribute.
pub(crate) fn has_sorted_attribute(expr_match: &ExprMatch) -> Option<usize> {
    if expr_match.attrs.is_empty() {
        return None
    }

    // Find an attribute with identifier "sorted"
    for (attr_index, attribute) in expr_match.attrs.iter().enumerate() {
        if let Meta::Path(path, ..) = &attribute.meta {
            if let Some(path_segment) = path.segments.first() {
                if path_segment.ident == "sorted" {
                    return Some(attr_index)
                }
            }
        }
    }

    None
}

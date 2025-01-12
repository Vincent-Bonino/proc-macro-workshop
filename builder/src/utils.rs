use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment, Type,
    TypePath,
};

/// Determine if a type is an option, and returns the inner type's identifier if so.
///
/// Pattern match the complex token tree part by part.
pub(crate) fn is_type_optional(ty: &Type) -> Option<&Ident> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        let path_segment: &PathSegment = segments.first()?;

        if path_segment.ident != "Option" {
            return None;
        }

        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &path_segment.arguments
        {
            if let GenericArgument::Type(Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            })) = args.first()?
            {
                let path_segment: &PathSegment = segments.first()?;

                return Some(&path_segment.ident);
            }
        }
    }

    None
}

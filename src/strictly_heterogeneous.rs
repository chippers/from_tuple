use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashSet;
use syn::{DeriveInput, Error, Field, Fields};

/// `impl` `From` for a tuple of field types in the order of the fields passed
///
/// If the field types are `String`, `u8`, and `i32`, then the generated `impl`
/// would be `impl From<(String, u8, i32)> for #struct` where `#struct` is the
/// `struct` you are deriving on.
pub(super) fn impl_from_tuple(fields: &[&Field], data: &DeriveInput) -> TokenStream2 {
    let struct_ident = &data.ident;
    let dvars = (0..fields.len())
        .map(|i| Ident::new(&format!("d{}", i), Span::call_site()))
        .collect::<Vec<_>>();

    let idents = fields.iter().map(|&f| f.ident.as_ref());
    let types = fields.iter().map(|&f| &f.ty);

    let tuple_type = quote! { (#(#types),*) };
    let destructed = quote! { (#(#dvars),*) };

    quote! {
        impl From<#tuple_type> for #struct_ident {

            #[inline]
            fn from(tuple: #tuple_type) -> Self {
                let #destructed = tuple;

                Self {
                    #(#idents: #dvars),*
                }
            }
        }
    }
}

/// Create spanned errors for every non-unique field type
pub(super) fn verify_unique_field_types(fields: &syn::Fields) -> syn::Result<()> {
    let mut seen = HashSet::new();
    let mut error = None;

    for field in fields {
        if !seen.insert(field.ty.clone()) {
            let new_error = Error::new_spanned(
                field,
                "Field types must be unique in a struct deriving `FromTuple`",
            );

            match error {
                None => error = Some(new_error),
                Some(ref mut error) => error.combine(new_error),
            }
        }
    }

    match error {
        None => Ok(()),
        Some(error) => Err(error),
    }
}

/// Pass all permutations of `syn::Fields` to a callback
///
/// Uses an iterative version of [`Heap's Algorithm`] to efficiently generate
/// all permutations.
///
/// [`Heap's Algorithm`]: https://en.wikipedia.org/wiki/Heap%27s_algorithm
pub(super) fn permute<F>(fields: &Fields, mut callback: F)
where
    F: FnMut(&[&Field]),
{
    let mut data = fields.iter().collect::<Vec<_>>();

    // the first permutation is just the unmodified field order
    callback(&data);

    let mut idx = 0;
    let mut stack = vec![0; data.len()];
    while idx < data.len() {
        if stack[idx] >= idx {
            stack[idx] = 0;
            idx += 1;
        } else {
            if idx % 2 == 0 {
                data.swap(0, idx);
            } else {
                data.swap(stack[idx], idx);
            }

            stack[idx] += 1;
            idx = 0;

            callback(&data);
        }
    }
}

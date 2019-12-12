//! Traits transforming types from tuples

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashSet;
use syn::{parse_macro_input, Data, DeriveInput, Error, Field, Fields};

/// Derive `From` tuples for `struct`s  that have unique field types.
///
/// Because of the restriction that field types must be unique, this derive
/// works best with structs that utilize [newtypes] for data.  Examples of
/// where this may be common is with http request parameters, or web form
/// inputs.
///
/// [newtypes]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
/// [`From`]: https://doc.rust-lang.org/core/convert/trait.From.html
///
/// # Example
///
/// ```
/// use from_tuple::FromTuple;
///
/// #[derive(FromTuple)]
/// struct Hello {
///     message: String,
///     time: i32,
///     counter: usize
/// }
///
/// fn main() {
///     let h1: Hello = ("world".into(), -1, 42usize).into();
///     assert_eq!(h1.time, -1);
///     assert_eq!(h1.counter, 42);
///     assert_eq!(&h1.message, "world");
///
///     let h2: Hello = (1_000_000_usize, i32::min_value(), "greetings".into()).into();
///     assert_eq!(h2.time, i32::min_value());
///     assert_eq!(h2.counter, 1_000_000);
///     assert_eq!(&h2.message, "greetings");
///
///     let h3: Hello = (-42, "hi".into(), 0usize).into();
///     assert_eq!(h3.time, -42);
///     assert_eq!(h3.counter, 0);
///     assert_eq!(&h3.message, "hi");
///
/// }
/// ```
///
/// ## Non-unique structs
///
/// Structs that have non-unique field types will fail to compile.  This is based
/// on the actual type, and not the alias, so it will fail on e.g. [`c_uchar`]
/// and [`u8`].
///
/// [`c_uchar`]: https://doc.rust-lang.org/std/os/raw/type.c_uchar.html
/// [`u8`]: https://doc.rust-lang.org/std/primitive.u8.html
///
/// ```compile_fail
/// use from_tuple::FromTuple;
///
/// #[derive(FromTuple)]
/// struct NonUnique {
///     first: String,
///     index: usize,
///     second: String,
/// }
/// ```
///
/// Attempting to compile the previous example will result in
///
/// ```bash
/// error: Field types must be unique in a struct deriving `FromTuple`
///   --> src/lib.rs:41:5
///    |
/// 10 |     second: String,
///    |     ^^^^^^^^^^^^^^
/// ```
///
/// ### Considerations
///
/// Support for non-unique types is under consideration for a future version,
/// but has not been implemented because it requires order-dependant fields for
/// structs - a *surprising* behaviour and can accidentally be broken by adding
/// a field in the wrong position unknowingly.
///
/// Requiring unique types may also be *surprising* behaviour, but is able to
/// be caught at compile time easily.  Additionally, I (personally) find it
/// less *surprising* than it being order-dependant.
#[proc_macro_derive(FromTuple)]
pub fn from_tuple(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data) = &input.data {
        if let Err(error) = verify_unique_field_types(&data.fields) {
            return error.to_compile_error().into();
        }

        let mut impls = Vec::new();
        permute(&data.fields, |fields| {
            impls.push(impl_from_tuple(fields, &input))
        });

        quote! { #(#impls)* }
    } else {
        Error::new_spanned(input, "FromTuple currently only supports Struct").to_compile_error()
    }
    .into()
}

/// `impl` `From` for a tuple of field types in the order of the fields passed
///
/// If the field types are `String`, `u8`, and `i32`, then the generated `impl`
/// would be `impl From<(String, u8, i32)> for #struct` where `#struct` is the
/// `struct` you are deriving on.
fn impl_from_tuple(fields: &[&Field], data: &DeriveInput) -> TokenStream2 {
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
fn verify_unique_field_types<'a>(fields: &syn::Fields) -> syn::Result<()> {
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
fn permute<F>(fields: &Fields, mut callback: F)
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

//! Traits transforming types from tuples

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashSet;
use syn::{parse_macro_input, Data, DeriveInput, Error, Field};

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
pub fn from_tuple(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derived = parse_macro_input!(input as DeriveInput);
    match do_from_tuple(derived) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

fn do_from_tuple(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut permutations = Vec::new();

    if let Data::Struct(r#struct) = input.data {
        check_for_unique_fields(r#struct.fields.iter())?;

        let mut idx = 0;
        let mut stack = vec![0; r#struct.fields.len()];
        let mut fields = r#struct.fields.iter().collect::<Vec<_>>();

        // the first permutation is just the unmodified field order
        permutations.push(impl_fields(&fields, &input.ident));

        // heap's algorithm for generating all permutations
        while idx < fields.len() {
            if stack[idx] < idx {
                if idx % 2 == 0 {
                    fields.swap(0, idx);
                } else {
                    fields.swap(stack[idx], idx);
                }

                stack[idx] += 1;
                idx = 0;

                permutations.push(impl_fields(&fields, &input.ident));
            } else {
                stack[idx] = 0;
                idx += 1;
            }
        }
    } else {
        return Err(Error::new_spanned(
            input,
            "FromTuple currently only supports Struct",
        ));
    }

    Ok(quote! { #(#permutations)* })
}

fn impl_fields(fields: &[&Field], impl_for: &Ident) -> TokenStream {
    // variables used for destructuring the tuple
    let dvars = (0..fields.len())
        .map(|i| Ident::new(&format!("d{}", i), Span::call_site()))
        .collect::<Vec<_>>();

    let idents = fields.iter().map(|&f| f.ident.as_ref());
    let types = fields.iter().map(|&f| &f.ty);

    let destruct = quote! { (#(#dvars), *) };
    let tuple = quote! { (#(#types),*) };
    quote! {
        impl From<#tuple> for #impl_for {
            #[inline]
            fn from(#destruct: #tuple) -> Self {
                Self {
                    #(#idents: #dvars),*
                }
            }
        }
    }
}

fn check_for_unique_fields<'a>(fields: impl Iterator<Item = &'a Field>) -> syn::Result<()> {
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

#[doc = include_str!("../README.md")]

#[cfg(any(feature = "strictly_heterogeneous", feature = "order_dependent"))]
use {
    proc_macro::TokenStream,
    quote::quote,
    syn::parse_macro_input,
};

#[cfg(feature="order_dependent")]
use {
    quote::ToTokens,
    proc_macro2::TokenStream as TokenStream2,
};

#[cfg(feature = "strictly_heterogeneous")]
mod strictly_heterogeneous;

#[cfg(feature = "strictly_heterogeneous")]
use {
    syn::{Data, DeriveInput, Error},
    strictly_heterogeneous::{impl_from_tuple, permute, verify_unique_field_types}
};

/// Derives `n!` implementations of [`core::convert::From<...>`][core::convert::From] on `struct`s that have 
/// unique field types `T1,T2,...,Tn`.
/// 
/// More precisely, derives implementations of [`core::convert::From<...>`][core::convert::From]
/// for all tuples-permuations of `T1,T2,...,Tn`, such as `(T1,T2,...,Tn-1,Tn)`, `(T1,T2,...,Tn,Tn-1)`,
/// and so on.
///
/// Because of the restriction that field types must be unique, this derive
/// works best with structs that utilize [newtypes] for data.  Examples of
/// where this may be common is with http request parameters, or web form
/// inputs.
///
/// [newtypes]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html
///
/// # Example
///
/// ```
/// use from_tuple::FromStrictlyHeterogeneousTuple;
///
/// #[derive(FromStrictlyHeterogeneousTuple)]
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
/// ## Structs with non-unique field types
///
/// Structs that have non-unique field types will fail to compile.  This is based
/// on the actual type, and not the alias, so it will fail on e.g. [`std::os::raw::c_uchar`]
/// and [`u8`].
///
/// ```compile_fail
/// use from_tuple::FromStrictlyHeterogeneousTuple;
///
/// #[derive(FromStrictlyHeterogeneousTuple)]
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
/// error: Field types must be unique in a struct deriving `FromStrictlyHeterogeneousTuple`
///   --> src/lib.rs:41:5
///    |
/// 10 |     second: String,
///    |     ^^^^^^^^^^^^^^
/// ```
///
/// ### [`FromStrictlyHeterogeneousTuple`] vs [`OrderDependentFromTuple`]
///
/// Dependence on order of fields in structs can be *surprising* behaviour as it may accidentally be broken by adding
/// a field in the wrong position unknowingly.
///
/// Requiring unique types may also be *surprising* behaviour, but is able to
/// be caught at compile time easily.
/// 
/// Also, at the moment of writing, only [`OrderDependentFromTuple`] also derives generic trait implementations
/// with the caveat that bounds must be only in the where clause.
#[cfg(feature = "strictly_heterogeneous")]
#[proc_macro_derive(FromStrictlyHeterogeneousTuple)]
pub fn from_strictly_heterogeneous_tuple(input: TokenStream) -> TokenStream {
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
        Error::new_spanned(input, "FromStrictlyHeterogeneousTuple currently only supports Struct").to_compile_error()
    }
    .into()
}

/// Derives implementation of [`core::convert::From<(T1,T2,...,Tn)>`][core::convert::From] on `struct`s
/// whose fields' types are `T1,T2,...,Tn`.
/// 
/// # Example
/// 
/// ```
/// use from_tuple::OrderDependentFromTuple;
/// 
/// #[derive(OrderDependentFromTuple)]
/// struct Hello {
///     offset: usize,
///     len: usize,
/// }
/// 
/// let strukt = Hello::from((234, 16));
/// assert_eq!(strukt.offset, 234);
/// assert_eq!(strukt.len, 16);
/// ```
/// 
/// ### [`FromStrictlyHeterogeneousTuple`] vs [`OrderDependentFromTuple`]
///
/// Dependence on order of fields in structs can be *surprising* behaviour as it may accidentally be broken by adding
/// a field in the wrong position unknowingly.
///
/// Requiring unique types may also be *surprising* behaviour, but is able to
/// be caught at compile time easily.
/// 
/// Also, at the moment of writing, only [`OrderDependentFromTuple`] also derives generic trait implementations
/// with the caveat that bounds must be only in the where clause
#[cfg(feature="order_dependent")]
#[proc_macro_derive(OrderDependentFromTuple)]
pub fn derive_from(item: TokenStream) -> TokenStream {
    use syn::{ItemStruct, Fields, token::Comma};

    let item_struct = parse_macro_input!(item as ItemStruct);
    let fields = match item_struct.fields {
        Fields::Named(fields) => fields,
        _ => panic!("expected named fields"),
    };

    let struct_name = item_struct.ident;
    let where_clause = item_struct.generics.where_clause.as_ref();
    let generics = &item_struct.generics;
    let fields_iter = fields.named.iter();
    let fields_tys_ts = fields_iter.clone()
        .map(|f| f.ty.clone())
        .fold(TokenStream2::new(), |mut ts,ty| {
            let ty_ts: TokenStream2 = ty.into_token_stream();
            ts.extend(ty_ts);
            let comma_ts = Comma::default().into_token_stream();
            ts.extend(comma_ts);
            ts
        });
    let fields_names_ts = fields_iter
        .filter_map(|f| f.ident.clone())
        .fold(TokenStream2::new(), |mut ts,ident| {
            let ident_ts: TokenStream2 = ident.into_token_stream();
            ts.extend(ident_ts);
            let comma_ts = Comma::default().into_token_stream();
            ts.extend(comma_ts);
            ts
        });


    let ts: TokenStream2 = quote! {
        impl #generics ::core::convert::From<(#fields_tys_ts)> for #struct_name #generics
        #where_clause {
            fn from((#fields_names_ts): (#fields_tys_ts)) -> Self {
                Self { #fields_names_ts }
            }
        }
    };
    ts.into()
}

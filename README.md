# from_tuple

[Derive macros] generating implementations of [`core::convert:From<...>`][`From`] on `struct`s.

Find more information on the documentation pages of [`FromStrictlyHeterogeneousTuple`] and [`OrderDependentFromTuple`].

## Examples

* [`FromStrictlyHeterogeneousTuple`]

```rust
use from_tuple::FromStrictlyHeterogeneousTuple;

#[derive(FromStrictlyHeterogeneousTuple)]
struct Hello {
    message: String,
    time: i32,
    counter: usize
}

let h1: Hello = ("world".into(), -1, 42usize).into();
assert_eq!(h1.time, -1);
assert_eq!(h1.counter, 42);
assert_eq!(&h1.message, "world");

let h2: Hello = (1_000_000_usize, i32::min_value(), "greetings".into()).into();
assert_eq!(h2.time, i32::min_value());
assert_eq!(h2.counter, 1_000_000);
assert_eq!(&h2.message, "greetings");

let h3: Hello = (-42, "hi".into(), 0usize).into();
assert_eq!(h3.time, -42);
assert_eq!(h3.counter, 0);
assert_eq!(&h3.message, "hi");
```

* [`OrderDependentFromTuple`]

```rust
use from_tuple::OrderDependentFromTuple;

#[derive(OrderDependentFromTuple)]
struct Hello {
    offset: usize,
    len: usize,
}

let strukt = Hello::from((234, 16));
assert_eq!(strukt.offset, 234);
assert_eq!(strukt.len, 16);
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

[Derive macros]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros
[`From`]: https://doc.rust-lang.org/nightly/core/convert/trait.From.html
[`FromStrictlyHeterogeneousTuple`]: https://docs.rs/from_tuple/latest/from_tuple/derive.FromStrictlyHeterogeneousTuple.html
[`OrderDependentFromTuple`]: https://docs.rs/from_tuple/latest/from_tuple/derive.OrderDependentFromTuple.html
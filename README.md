# from_tuple

[Derive macros] generating implementations of `core::convert:From<...>` on `struct`s.

Find more information on the documentation pages of [`FromStrictlyHeterogeneousTuple`] and [`OrderDependentFromTuple`].

[`FromStrictlyHeterogeneousTuple`]: https://docs.rs/from_tuple/latest/from_tuple/derive.FromStrictlyHeterogeneousTuple.html
[`OrderDependentFromTuple`]: https://docs.rs/from_tuple/latest/from_tuple/derive.OrderDependentFromTuple.html

## Example

```rust
use from_tuple::FromStrictlyHeterogeneousTuple;

#[derive(FromStrictlyHeterogeneousTuple)]
struct Hello {
    message: String,
    time: i32,
    counter: usize
}

fn main() {
    let hello: Hello = (-42, "hi".to_string(), 0usize).into();

    assert_eq!(&hello.message, "hi");
    assert_eq!(hello.time, -42);
    assert_eq!(hello.counter, 0);
}
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
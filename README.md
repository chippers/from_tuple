# from_tuple

Derive `From<Tuple>` for `struct`s  that have unique field types.

Because of the restriction that field types must be unique, this derive
works best with structs that utilize [newtypes] for data.  Examples of
where this may be common is with http request parameters, or web form
inputs.

[newtypes]: https://doc.rust-lang.org/rust-by-example/generics/new_types.html

## Example

```rust
use from_tuple::FromTuple;

#[derive(FromTuple)]
struct Hello {
    message: String,
    time: i32,
    counter: usize
}

fn main() {
    let h1: Hello = ("world".into(), -1, 42usize).into();
    let h2: Hello = (1_000_000_usize, i32::min_value(), "greetings".into()).into();
    let h3: Hello = (-42, "hi".into(), 0usize).into();

    assert_eq!(h1.time, -1);
    assert_eq!(h2.time, i32::min_value());
    assert_eq!(h3.time, -42);

    assert_eq!(h1.counter, 42);
    assert_eq!(h2.counter, 1_000_000);
    assert_eq!(h3.counter, 0);

    assert_eq!(&h1.message, "world");
    assert_eq!(&h2.message, "greetings");
    assert_eq!(&h3.message, "hi");
}
```

### Non-unique structs

Structs that have non-unique field types will fail to compile.  This is based
on the actual type, and not the alias, so it will fail on e.g. [`c_uchar`]
and [`u8`].

[`c_uchar`]: https://doc.rust-lang.org/std/os/raw/type.c_uchar.html
[`u8`]: https://doc.rust-lang.org/std/primitive.u8.html

```compile_fail
use from_tuple::FromTuple;

#[derive(FromTuple)]
struct NonUnique {
    first: String,
    index: usize,
    second: String,
}
```

Attempting to compile the previous example will result in

```bash
error: Field types must be unique in a struct deriving `FromTuple`
  --> src/lib.rs:41:5
   |
10 |     second: String,
   |     ^^^^^^^^^^^^^^
```

#### Considerations

Support for non-unique types is under consideration for a future version,
but has not been implemented because it requires order-dependant fields for
structs - a *surprising* behaviour and can accidentally be broken by adding
a field in the wrong position unknowingly.

Requiring unique types may also be *surprising* behaviour, but is able to
be caught at compile time easily.  Additionally, I (personally) find it
less *surprising* than it being order-dependant.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

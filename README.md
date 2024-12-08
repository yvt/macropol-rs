# 🚨 macropol

Ergonomic string literal interpolation in macro definitions.

Replaces metavariables (`$foo`) and arbitrary expressions in string literals (including doc comments) and concatenates them with surrounding text fragments with `core::concat!`.

```rust
#[macropol::macropol]
macro_rules! mymacro {
    ($count:expr, $name:literal, fn $func:ident()) => {
        /// Returns `"$$ $name, $&count to beam up"`.
        fn $func() -> &'static str {
            "$$ $name, ${stringify!($count)} to beam up"
        }
    };
}

// The above definition expands to:
//
//     macro_rules! mymacro {
//         ($count:expr, $name:expr, fn $func:ident()) => {
//             #[doc = concat!("Returns `\"$ ", $name, ", ",
//                  stringify!($count), " to beam up\"`.")]
//             fn func() -> &'static str {
//                 concat!("$ ", $name, ", ",
//                     stringify!($count), " to beam up")
//             }
//         };
//     }
//

mymacro!(3, "Scotty", fn func());

assert_eq!(func(), "$ Scotty, 3 to beam up");
```

//! `unstringify!` (https://crates.io/crates/unstringify) is a proc-macro to
//! parse the provided string literals. It can partially simulate eager
//! expansion by recognizing certain macro names, such as `concat!` and
//! `stringify!`. In the default configuration, `macropol` uses `concat!`'s
//! fully-qualified path `::core::concat!`, which does not get recognized by
//! `unstringify!`. This test demonstrates the use of the `concat!` parameter to
//! address this issue.

#[macropol::macropol(concat = "concat!($parts_comma_sep)")]
macro_rules! m {
    ($t:literal) => {
        unstringify::unstringify!("1 * $t * 2")
    };
}

#[test]
fn test() {
    assert_eq!(m!("3 + 4"), (1 * 3 + 4 * 2));
}

//! Utilities only used in crate tests.

/// Make sure that an expression returns a specific error value, panicking otherwise.
///
/// The first argument is any expression that evaluates to [`Result`]. The second argument is any
/// pattern that could fit inside an [`Err`] value- this is the value you expect to be returned in
/// the error.
#[macro_export]
macro_rules! assert_err {
    ($exp:expr, $err:pat) => {
        match $exp {
            Err($err) => {} // OK!
            val => panic!("expected Err({}), got {val:?}", stringify!($err)),
        }
    };
}

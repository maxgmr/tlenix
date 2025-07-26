//! This module contains various functions to abstract away number-related things that would
//! otherwise be a pain to implement oneself.

/// Gets the number of digits in the given `usize`, when interpreted as base 10.
#[must_use]
pub fn num_digits_base10(n: usize) -> usize {
    // The fir tree of doom!!
    //
    // This is, unfortunately, the highest-performance way I can think of doing this :(
    if n < 10 {
        1
    } else if n < 100 {
        2
    } else if n < 1_000 {
        3
    } else if n < 10_000 {
        4
    } else if n < 100_000 {
        5
    } else if n < 1_000_000 {
        6
    } else if n < 10_000_000 {
        7
    } else if n < 100_000_000 {
        8
    } else if n < 1_000_000_000 {
        9
    } else if n < 10_000_000_000 {
        10
    } else if n < 100_000_000_000 {
        11
    } else if n < 1_000_000_000_000 {
        12
    } else if n < 10_000_000_000_000 {
        13
    } else if n < 100_000_000_000_000 {
        14
    } else if n < 1_000_000_000_000_000 {
        15
    } else if n < 10_000_000_000_000_000 {
        16
    } else if n < 100_000_000_000_000_000 {
        17
    } else if n < 1_000_000_000_000_000_000 {
        18
    } else if n < 10_000_000_000_000_000_000 {
        19
    } else {
        20
    }
}

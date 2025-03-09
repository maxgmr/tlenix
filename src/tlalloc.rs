//! Tlalloc, Tlenix's custom memory allocator.

use lazy_static::lazy_static;

use crate::{
    data::NullTermStr,
    eprintln,
    fs::{OpenFlags, open_no_create, read},
    nulltermstr,
};

const AUXV_PATH: NullTermStr<16> = nulltermstr!(b"/proc/self/auxv");

const PAGE_SIZE_FALLBACK: usize = 4092;

lazy_static! {
    /// The page size of the system.
    pub static ref PAGE_SIZE: usize = page_size();
}

/// Get the system page size.
fn page_size() -> usize {
    let mut buf = [0_u8; 16];
    let Ok(fd) = open_no_create(&AUXV_PATH, &OpenFlags::O_RDONLY) else {
        // Failed to open auxiliary vector :(
        return fallback_page_size();
    };

    let mut auxv_entry: [usize; 2] = [0; 2];

    while read(fd, &mut buf) == Ok(16) {
        auxv_entry[0] = usize::from_ne_bytes(buf[0..8].try_into().unwrap());
        auxv_entry[1] = usize::from_ne_bytes(buf[8..16].try_into().unwrap());
        if auxv_entry[0] == 6 {
            return auxv_entry[1]; // AT_PAGESZ entry
        }
    }
    // Didn't find AT_PAGESZ :(
    fallback_page_size()
}

fn fallback_page_size() -> usize {
    // Unable to get access to `auxv`, forced to use fallback.
    #[cfg(debug_assertions)]
    eprintln!(
        "Warning: failed to get page size from /proc/self/auxv; forced to use fallback value {}",
        PAGE_SIZE_FALLBACK
    );
    PAGE_SIZE_FALLBACK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::println;

    #[test_case]
    fn page_size() {
        println!("{}", page_size());
    }
}

//! Tlalloc, Tlenix's custom memory allocator.

use lazy_static::lazy_static;

use crate::{Errno, data::NullTermStr, eprintln, fs::read_from_file, nulltermstr};

const AUXV_PATH: NullTermStr<16> = nulltermstr!(b"/proc/self/auxv");

const PAGE_SIZE_FALLBACK: usize = 4092;

lazy_static! {
    /// The page size of the system.
    pub static ref PAGE_SIZE: usize = page_size();
}

/// Get the system page size.
fn page_size() -> usize {
    let mut auxv_entry: [usize; 2] = [0; 2];
    loop {
        let Ok(buf): Result<[u8; 16], Errno> = read_from_file(&AUXV_PATH) else {
            return fallback_page_size();
        };

        if buf[15] == b'\0' {
            return fallback_page_size();
        }

        auxv_entry[0] = usize::from_ne_bytes(buf[0..8].try_into().unwrap());
        auxv_entry[1] = usize::from_ne_bytes(buf[8..16].try_into().unwrap());
        if auxv_entry[0] == 6 {
            // AT_PAGESZ entry!
            return auxv_entry[1];
        }
    }
}

fn fallback_page_size() -> usize {
    // Unable to get access to `auxv`, forced to use fallback.
    #[cfg(debug_assertions)]
    eprintln!(
        "Warning: forced to use fallback value ({})",
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

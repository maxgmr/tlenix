//! The global memory allocator.

use core::ptr;

use talc::{ClaimOnOom, Span, Talc, Talck};

// Size (in bytes) of global memory allocator arena.
const ARENA_SIZE: usize = 1 << 16; // 64 KiB

// Talc global memory allocator
static mut ARENA: [u8; ARENA_SIZE] = [0; ARENA_SIZE];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array(ptr::addr_of!(ARENA).cast_mut())) }).lock();

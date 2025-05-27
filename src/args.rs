//! Module for handling command-line arguments passed to
//! [`execve`](https://man7.org/linux/man-pages/man2/execve.2.html)-compatible binaries.

use alloc::{string::String, vec::Vec};
use core::slice;

use crate::{ARG_ENV_LIM, ARG_LEN_LIM, ENV_LEN_LIM, Errno, NULL_BYTE};

/// Environment variables parsed from the stack using Linux `execve` conventions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvVar {
    key: String,
    value: String,
}
impl TryFrom<String> for EnvVar {
    type Error = Errno;

    fn try_from(mut value: String) -> Result<Self, Self::Error> {
        if let Some(eq_idx) = value.find('=') {
            let v = value.split_off(eq_idx);
            Ok(Self {
                key: value,
                value: v,
            })
        } else {
            Err(Errno::Einval)
        }
    }
}

/// Parses `argv` and `envp` from the stack.
///
/// # Errors
///
/// This function returns an [`Errno`] in the following cases:
///
/// - [`Errno::Eilseq`]: The provided bytes are not valid UTF-8.
/// - [`Errno::E2big`]: The provided argument list is too long.
/// - [`Errno::Einval`]: `argc` does not match the actual number of arguments in `argv`.
///
/// # Safety
///
/// This function reads whatever happens to be at the provided stack pointer and validates input as
/// best as it can, but safety cannot be guaranteed considering the fact that it's simply reading
/// bytes directly starting from the provided pointer.
///
/// Here are some ways you can call this function to make it _less_ horribly unsafe:
///
/// - Call this function right at the entry point of your binary.
/// - Make sure the provided pointer is _actually_ pointing to the top of the stack!
#[allow(clippy::similar_names)]
pub unsafe fn parse_argv_envp(
    stack_ptr: *const usize,
) -> Result<(Vec<String>, Vec<EnvVar>), Errno> {
    // Keep track of the total size of `argv` and `envp`
    let mut total_size: usize = 0;

    // Argc is the first `usize`
    let argc: usize = unsafe { *stack_ptr };

    // Go past `argc` to reach the start of `argv` and start reading the raw bytes
    let mut ptr = unsafe { stack_ptr.add(1).cast::<*const u8>() };

    // Start parsing argv[0..argc]
    let mut argv = Vec::with_capacity(argc);
    for _ in 0..argc {
        let arg_ptr = unsafe { *ptr };
        if arg_ptr.is_null() {
            // argc does not match argv!
            return Err(Errno::Einval);
        }

        // Figure out the length of this arg
        // SAFETY: A limit to the argument length is set, returning `Err` if it's too long.
        let len = unsafe {
            slice::from_raw_parts(arg_ptr, ARG_LEN_LIM)
                .iter()
                .position(|&byte| byte == NULL_BYTE)
                .ok_or(Errno::E2big)?
        };
        total_size = inc_total_size(total_size, len)?;

        // SAFETY: The length has been calculated to end at the null byte.
        let arg_string: String = unsafe {
            String::from_utf8(slice::from_raw_parts(arg_ptr, len).to_vec())
                .map_err(|_| Errno::Eilseq)?
        };
        argv.push(arg_string);

        // Advance the pointer to point to the next `argv`.
        ptr = unsafe { ptr.add(1) };
    }

    // Double check to make sure we're pointing to the null terminator of argv.
    if unsafe { !(*ptr).is_null() } {
        // argc does not match argv!
        return Err(Errno::Einval);
    }

    // Advance pointer to envp
    ptr = unsafe { ptr.add(1) };

    // Start parsing envp
    let mut envp = Vec::new();
    loop {
        let env_ptr = unsafe { *ptr };
        if env_ptr.is_null() {
            break;
        }
        let len = unsafe {
            slice::from_raw_parts(env_ptr, ENV_LEN_LIM)
                .iter()
                .position(|&byte| byte == NULL_BYTE)
                .ok_or(Errno::E2big)?
        };
        total_size = inc_total_size(total_size, len)?;

        // SAFETY: The length has been calculated to end at the null byte.
        let env_base_string: String = unsafe {
            String::from_utf8(slice::from_raw_parts(env_ptr, len).to_vec())
                .map_err(|_| Errno::Eilseq)?
        };
        envp.push(EnvVar::try_from(env_base_string)?);

        // Advance the pointer to point to the next `envp`.
        ptr = unsafe { ptr.add(1) };
    }

    Ok((argv, envp))
}

fn inc_total_size(total_size: usize, increase: usize) -> Result<usize, Errno> {
    let result = total_size + increase;
    if result > ARG_ENV_LIM {
        Err(Errno::E2big)
    } else {
        Ok(result)
    }
}

//! Module for handling command-line arguments passed to
//! [`execve`](https://man7.org/linux/man-pages/man2/execve.2.html)-compatible binaries.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::slice;

use crate::{ARG_ENV_LIM, ARG_LEN_LIM, ENV_LEN_LIM, Errno, NULL_BYTE};

/// Character separating the value of an [`EnvVar`] from its key.
const ENV_VAR_SEPARATOR: char = '=';

/// Environment variables parsed from the stack using Linux `execve` conventions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvVar {
    /// The key of the environment variable.
    pub key: String,
    /// The value of the environment variable.
    pub value: String,
}
impl TryFrom<String> for EnvVar {
    type Error = Errno;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        if let Some(eq_idx) = string.find(ENV_VAR_SEPARATOR) {
            // Can't have an empty key!
            if eq_idx == 0 {
                return Err(Errno::Einval);
            }
            // SAFETY: We know `eq_idx` is within bounds. `find` returned a valid index.
            let key = string[..eq_idx].to_string();
            let value = string[eq_idx + 1..].to_string();
            Ok(Self { key, value })
        } else {
            Err(Errno::Einval)
        }
    }
}
impl TryFrom<&String> for EnvVar {
    type Error = Errno;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}
impl TryFrom<&str> for EnvVar {
    type Error = Errno;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}
impl From<EnvVar> for String {
    fn from(value: EnvVar) -> Self {
        Self::from(&value)
    }
}
impl From<&EnvVar> for String {
    fn from(value: &EnvVar) -> Self {
        let total_len = value.key.len() + value.value.len() + 1;
        let mut string = String::with_capacity(total_len);
        string.push_str(&value.key);
        string.push(ENV_VAR_SEPARATOR);
        string.push_str(&value.value);
        string
    }
}
impl core::fmt::Display for EnvVar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_err;

    macro_rules! test_ev_from {
        ($fn_name:ident($input:expr) => OK($key:expr, $value:expr)) => {
            #[test_case]
            fn $fn_name() {
                let input = $input;
                let result = EnvVar::try_from(input).unwrap();
                assert_eq!(
                    result,
                    EnvVar {
                        key: $key.to_string(),
                        value: $value.to_string()
                    }
                );
            }
        };
        ($fn_name:ident($input:expr) => ERR($e:pat)) => {
            #[test_case]
            fn $fn_name() {
                let input = $input;
                $crate::assert_err!(EnvVar::try_from(input), $e);
            }
        };
    }
    test_ev_from!(ev_from_string("MY_KEY=my_val".to_string()) => OK("MY_KEY", "my_val"));
    test_ev_from!(ev_with_space("NAME=Maxwell Gilmour".to_string()) => OK("NAME", "Maxwell Gilmour"));
    test_ev_from!(ev_no_eq("MY_KEY my_val".to_string()) => ERR(Errno::Einval));
    test_ev_from!(ev_from_str("MY_KEY=123") => OK("MY_KEY", "123"));
    test_ev_from!(ev_empty_key("=my_val".to_string()) => ERR(Errno::Einval));
    test_ev_from!(ev_empty_val("MY_KEY=".to_string()) => OK("MY_KEY", ""));
    test_ev_from!(ev_multibyte("我的叫=马克斯".to_string()) => OK("我的叫", "马克斯"));

    #[test_case]
    fn inc_total_size_under() {
        assert_eq!(inc_total_size(1, 1), Ok(2));
        assert_eq!(inc_total_size(ARG_ENV_LIM - 2, 2), Ok(ARG_ENV_LIM));
    }

    #[test_case]
    fn inc_total_size_over() {
        assert_err!(inc_total_size(ARG_ENV_LIM, 1), Errno::E2big);
    }
}

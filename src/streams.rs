//! Module for the [standard streams](https://en.wikipedia.org/wiki/Standard_streams): standard
//! input, standard output, and standard error.

use alloc::{string::String, vec::Vec};
use core::marker::PhantomData;

use spin::Mutex;

use crate::{
    Errno,
    fs::{File, FileDescriptor},
    term::{
        ControlCharIndex, ControlModeFlags, InputModeFlags, LocalModeFlags, OutputModeFlags,
        SetTermAttrsCmd, Termios, WinSize, get_term_attrs, get_term_size, set_term_attrs,
    },
};

/// File descriptor of the standard input stream.
const STDIN_FILENO: usize = 0;
/// File descriptor of the standard output stream.
const STDOUT_FILENO: usize = 1;
/// File descriptor of the standard error stream.
const STDERR_FILENO: usize = 2;

/// The
/// [standard input stream](https://en.wikipedia.org/wiki/Standard_streams#Standard_input_(stdin)),
/// from which programs can read input data.
pub static STDIN: Mutex<Stream<Input>> = Mutex::new(Stream::define(STDIN_FILENO));
/// The
/// [standard output stream](https://en.wikipedia.org/wiki/Standard_streams#Standard_output_(stdout)),
/// to which programs can write output data.
pub static STDOUT: Mutex<Stream<Output>> = Mutex::new(Stream::define(STDOUT_FILENO));
/// The
/// [standard error stream](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)),
/// to which programs can write error messages or diagnostics.
pub static STDERR: Mutex<Stream<Output>> = Mutex::new(Stream::define(STDERR_FILENO));

/// An input stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Input;
/// An output stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Output;

/// Macro to implement the getter and setter [`Termios`] methods for all the mode flags.
macro_rules! impl_termios_flags_methods {
    [$($flags_t:ty),* $(,)?] => {
        $(paste::paste! {
            /// Gets the value of the given
            #[doc = concat!("[`", stringify!($flags_t), "`]")]
            /// flag.
            ///
            /// If multiple flags are given, then this function will only return `true` if *all*
            /// the given flags are set.
            ///
            /// # Errors
            ///
            /// This function propagates any [`Errno`]s incurred by the underlying call to
            /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html).
            pub fn [<get_ $flags_t:snake>](&self, flag: $flags_t) -> Result<bool, Errno> {
                let termios = self.termios()?;
                Ok(termios.[<$flags_t:snake>].contains(flag))
            }

            /// Sets the value of the given
            #[doc = concat!("[`", stringify!($flags_t), "`]")]
            /// flag to the given boolean value.
            ///
            /// If multiple flags are given, then *all* given flags will be set to the given
            /// boolean value.
            ///
            /// The given [`SetTermAttrsCmd`] dictates the following:
            ///
            /// - Whether or not the system waits to apply changes after all written output has
            /// been transmitted.
            /// - Whether or not the system discards unread input before applying changes.
            ///
            /// # Errors
            ///
            /// This function propagates any [`Errno`]s incurred by the underlying calls to
            /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html).
            pub fn [<set_ $flags_t:snake>](
                &mut self,
                cmd: SetTermAttrsCmd,
                flag: $flags_t,
                value: bool
            ) -> Result<(), Errno> {
                let mut termios = self.termios()?;
                termios.[<$flags_t:snake>].set(flag, value);
                self.set_termios(cmd, &termios)
            }
        })*
    };
}

/// A [`File`] corresponding to a particular
/// [`standard stream`](https://en.wikipedia.org/wiki/Standard_streams).
#[derive(Debug, PartialEq, Hash)]
pub struct Stream<D> {
    file: File,
    direction: PhantomData<D>,
}
impl<D> Stream<D> {
    /// Statically define the [`FileDescriptor`] corresponding to this standard stream, and whether
    /// the stream is an input stream or an output stream.
    const fn define(raw_fd: usize) -> Self {
        Self {
            file: File::define(FileDescriptor::define(raw_fd)),
            direction: PhantomData,
        }
    }
}
impl Stream<Input> {
    /// Reads bytes from the stream into the given buffer. Returns the number of bytes read from
    /// the stream on success.
    ///
    /// Wrapper around the [`File::read`] function.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from [`File::read`].
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, Errno> {
        self.file.read(buffer)
    }

    /// Reads the entire stream, up to EOF, into a [`Vec<u8>`].
    ///
    /// Wrapper around the [`File::read_to_bytes`] function.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from [`File::read_to_bytes`].
    pub fn read_to_bytes(&self) -> Result<Vec<u8>, Errno> {
        self.file.read_to_bytes()
    }

    /// Reads the entire stream, up to EOF, into a [`String`].
    ///
    /// Wrapper around the [`File::read_to_string`] function.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from [`File::read_to_string`].
    pub fn read_to_string(&self) -> Result<String, Errno> {
        self.file.read_to_string()
    }

    /// Gets the state of the current terminal, in the form of
    /// [`termios(3)`](https://www.man7.org/linux/man-pages/man3/termios.3.html).
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from the underlying
    /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html) syscall.
    pub fn termios(&self) -> Result<Termios, Errno> {
        get_term_attrs(self.file.fd_raw())
    }

    /// Sets the state of the current terminal to the given [`Termios`].
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from the underlying
    /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html) syscalls.
    pub fn set_termios(&mut self, cmd: SetTermAttrsCmd, termios: &Termios) -> Result<(), Errno> {
        set_term_attrs(cmd, self.file.fd_raw(), termios)
    }

    /// Gets the value of the given control character.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from the underlying
    /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html) syscall.
    pub fn control_character(&self, ctrl_char: ControlCharIndex) -> Result<u8, Errno> {
        let termios = self.termios()?;
        // OK to index here- the ControlCharIndex enum restricts the index to valid values.
        Ok(termios.control_characters[ctrl_char as usize])
    }

    /// Sets the value of the given control character.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from the underlying
    /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html) syscall.
    pub fn set_control_character(
        &mut self,
        cmd: SetTermAttrsCmd,
        ctrl_char: ControlCharIndex,
        value: u8,
    ) -> Result<(), Errno> {
        let mut termios = self.termios()?;
        // OK to index here- the ControlCharIndex enum restricts the index to valid values.
        termios.control_characters[ctrl_char as usize] = value;
        self.set_termios(cmd, &termios)
    }

    /// Gets the window size of the terminal.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from the underlying
    /// [`ioctl(2)`](https://man7.org/linux/man-pages/man2/ioctl.2.html) syscall.
    pub fn win_size(&self) -> Result<WinSize, Errno> {
        get_term_size(self.file.fd_raw())
    }

    impl_termios_flags_methods![
        InputModeFlags,
        OutputModeFlags,
        ControlModeFlags,
        LocalModeFlags,
    ];
}
impl Stream<Output> {
    /// Writes bytes from the provided buffer into the stream, returning the number of bytes
    /// written.
    ///
    /// Wrapper around the [`File::write`] function.
    ///
    /// # Errors
    ///
    /// This function propagates any [`Errno`]s returned from [`File::write`].
    pub fn write(&self, buffer: &[u8]) -> Result<usize, Errno> {
        self.file.write(buffer)
    }
}
impl core::fmt::Write for Stream<Output> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).map_err(|_| core::fmt::Error {})?;
        Ok(())
    }
}

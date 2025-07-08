//! This module contains everything related to the terminal window size.

/// The width and height of the terminal window, in both characters and pixels.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WinSize {
    /// The width of the terminal in characters.
    pub rows: usize,
    /// The height of the terminal in characters.
    pub cols: usize,
    /// The width of the terminal in pixels.
    pub width: usize,
    /// The heigh of the terminal in pixels.
    pub height: usize,
}
impl From<WinSizeRaw> for WinSize {
    fn from(value: WinSizeRaw) -> Self {
        Self {
            rows: value.row as usize,
            cols: value.col as usize,
            width: value.xpixel as usize,
            height: value.ypixel as usize,
        }
    }
}

/// The raw terminal window size, retrieved directly from the `ioctl` syscall.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct WinSizeRaw {
    row: u16,
    col: u16,
    xpixel: u16,
    ypixel: u16,
}

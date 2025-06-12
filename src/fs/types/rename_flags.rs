//! Module for the [`RenameFlags`] type.

bitflags::bitflags! {
    /// The options which can be passed to the [`crate::fs::rename`] function.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RenameFlags: u32 {
        /// Don't overwrite the destination. Instead, return an error if the destination already
        /// exists. Incompatible with [`RenameFlags::EXCHANGE`].
        const NOREPLACE = 1;
        /// Automatically exchange the new and old paths. Incompatible with
        /// [`RenameFlags::NOREPLACE`].
        const EXCHANGE = 2;
        /// Atomically create a "whiteout" object at the source path while performing the rename.
        const WHITEOUT = 4;
    }
}
impl Default for RenameFlags {
    fn default() -> Self {
        Self::empty()
    }
}

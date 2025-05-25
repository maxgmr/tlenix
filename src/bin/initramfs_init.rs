//! This executable is packed into `initramfs` and executed by the kernel as PID1. It performs the
//! following steps:
//!
//! 1. Mounts essential filesystems
//! 2. Finds and mounts the rootfs
//! 3. Switches to the real rootfs

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic
)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![cfg_attr(test, test_runner(tlenix_core::custom_test_runner))]

use core::{panic::PanicInfo, time::Duration};

use tlenix_core::{
    ExitStatus, align_stack_pointer, eprintln,
    fs::{self, MountFlags},
    process, thread,
};

/// The name of the process for the purposes of the panic message.
const INITRAMFS_INIT_PANIC_TITLE: &str = "initramfs_init";

/// Path to the new root mount point.
const NEWROOT: &str = "/newroot";

/// Path to the ext4 root partition device.
const EXT4_DEV: &str = "/dev/sda2";

/// Path to the real init program on the new root.
const REAL_INIT: &str = "/sbin/init";

/// Directory that becomes the mount point of the old initramfs root.
const OLDROOT: &str = "/newroot/oldroot";

/// Filesystem permissions for new directories.
const DIR_MODE_755: usize = 0o755;

/// Entry point.
///
/// # Panics
///
/// This function panics on any failure to bring up the root filesystem, mount any filesystems, or
/// run the real init.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    use fs::FilesystemType::{Devtmpfs, Ext4, Proc, Sysfs};

    align_stack_pointer!();

    #[cfg(any(test, debug_assertions))]
    process::exit(ExitStatus::ExitSuccess);

    // HACK: This stops the compiler from complaining when building the test/debug target
    #[allow(unreachable_code)]
    #[allow(clippy::no_effect)]
    ();

    let dir_mode = fs::FilePermissions::from_bits(DIR_MODE_755).unwrap();

    // Mount virtual filesystems
    fs::mount("none", "/proc", Proc, fs::MountFlags::default()).unwrap();
    fs::mount("none", "/sys", Sysfs, fs::MountFlags::default()).unwrap();
    fs::mount("none", "/dev", Devtmpfs, fs::MountFlags::default()).unwrap();

    // Mount persistent root filesystem
    fs::mkdir(NEWROOT, dir_mode).unwrap();
    fs::mount(EXT4_DEV, NEWROOT, Ext4, MountFlags::empty()).unwrap();

    // Prepare oldroot directory (must be a mount point)
    fs::mkdir(OLDROOT, dir_mode).unwrap();

    // chdir into new root
    fs::change_dir(NEWROOT).unwrap();

    // Make oldroot a mount point
    fs::mount(
        "oldroot",
        "oldroot",
        fs::FilesystemType::Bind,
        MountFlags::MS_BIND,
    )
    .unwrap();

    // Escape rootfs
    fs::chroot(".").unwrap();

    // Switch to new '/'
    fs::change_dir("/").unwrap();

    // Clean up old root
    fs::umount("/oldroot", fs::UmountFlags::MNT_DETACH).unwrap();
    fs::rmdir("/oldroot").unwrap();

    // Start real init
    if process::execve(&[REAL_INIT], &[""; 0]).is_err() {
        eprintln!("Failed to start '{REAL_INIT}'; exiting in 5 seconds");
        thread::sleep(&Duration::from_secs(5)).unwrap();
        process::exit(ExitStatus::ExitFailure);
    }
    unreachable!("execve replaces the process; we should not return");
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{INITRAMFS_INIT_PANIC_TITLE} {info}");
    process::exit(ExitStatus::ExitFailure)
}

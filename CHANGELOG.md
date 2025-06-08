# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Planned]

- Custom global allocator.
- `mash` prompt should react to exit code of last-executed process.
- Fun BSOD-style screen on panic.
- Nicer interface for ANSI colour escape codes (and coloured printing in general).
- Get `mash` to "bundle together" stuff with quotes and generally handle quotes properly.
- Build script updates GRUB entry version number automatically
- Handle non-`/dev/sda{1,2}` configurations in `usb-setup` and `initramfs_init`
- Abstract away Tlenix program boilerplate (proc macro?)
- Add `--help` display to all Tlenix programs
- `man` pages!

## [0.1.0-beta.3] - UNRELEASED

### Added

- `cat`: Concatenate files and print on the standard output.
- `clear`: Clears the terminal screen.
- `File::read_to_bytes`: Reads the whole file into a byte vector.
- `format!`: Just like the `std` macro!
- `rename`: Changes the name/path of a file/directory.

### Changed

- Modernized file stats format (`fstat` -> `statx`).
- Statically access the standard input, standard output, and standard error streams.
- Prettier test output alignment.
- Fixed `./usb-install` file schema.
- `README` instructions now use `./usb-install` script.

## [0.1.0-beta.2] - 2025-05-27

### Added

- `hello`: Minimal demo Tlenix program. Useful as a template/example.
- `printenv`: Prints the environment variables.

### Removed

- Built-in `pwd` from `mash` (to be later implemented as a standalone program)
- `vec_into_nix_bytes` and `vec_into_nix_strings` (obsolete)

## [0.1.0-beta.1] - 2025-05-26

### Added

- Fully-bootable USB-based system on real hardware!
- `ls`: List entries within a directory.
- `mash` can now execute programs.
- Linux kernel custom configuration in `config/.config`.
- GRUB custom configuration in `config/grub.cfg`.
- Nicer terminal font.
- Pretty logo on boot.
- USB installation script (`usb-install`).

### Changed

- Overhauled `ExitStatus` to make it more expressive.
- Proper `execute_process` error reporting.

### Removed

- `mk-release-bins` script. Superseded by `usb-install`.

## [0.1.0-alpha.9] - 2025-05-24

### Added

- `initramfs_init`, an `init` program specifically for the `initramfs`.
- Create and remove directories (`mkdir`, `rmdir`).
- Delete files (`rm`).
- Get entries of a directory (`File::dir_ents`).
- Change the root mount (`pivot_root`).
- Execute a program (`execve`).
- Directly set the mode of a file when creating one (`OpenOptions::set_mode`).
- Read a file directly into a `String` (`File::read_to_string`).
- Change process root directory (`chroot`).

### Changed

- Restricted raw syscalls to crate only.
- Increased heap size from 16 KiB to 64 KiB.

## [0.1.0-alpha.8.1] - 2025-05-22

### Changed

- Fixed custom target issues causing segfaults in emulators.

## [0.1.0-alpha.8] - 2025-05-21

### Added

- `mount` and `umount` functions.
- `init` now tries to mount `/proc` and `/sys`.

## [0.1.0-alpha.7] - 2025-05-20

### Added

- Coloured test output.
- `NixString` and `NixBytes`. Null-terminated byte vectors compatible with Linux syscalls.
- `File` type. Provides filesystem operations on files. Hides the file descriptor and closes the file when dropped.
- `OpenOptions` type. Allows easy customization of `File` open flags. Guarantees safe open flag combinations when opening a file.
- Ability to create files with defined permissions.
- `SyscallArg` type, allowing for more flexible arguments for `syscall!` and `syscall_result!`
- Pretty colours in tests!

### Changed

- General rewrite of codebase.
- Interface for filesystem operations- now, any file-based operations must be called as `File` methods.

### Removed

- `mash`'s ability to execute programs. This will be re-implemented later.
- `NullTermString` and `NullTermStr`.
- `read_from_file()`.
- `open_no_create()`.
- `change_program_break()`.

## [0.1.0-alpha.6] - 2025-03-09

### Added

- Heap-based allocation and data types.
- `mash` process execution.

### Changed

- Tests now return with success or failure.

## [0.1.0-alpha.5] - 2025-03-08

### Added

- Colourful `mash` prompt.
- Show basename in `mash` prompt.
- `poweroff` command.
- `reboot` command.

## [0.1.0-alpha.4] - 2025-03-07

### Added

- `mash`: Basic shell which only echos (for now!)

## [0.1.0-alpha.3] - 2025-03-06

### Added

- Thread sleep function.

### Changed

- Improved endless loop that wastes fewer CPU cycles.

## [0.1.0-alpha.2] - 2025-03-06

### Added

- Read files.

## [0.1.0-alpha.1] - 2025-03-05

### Added

- Initial release.
- Welcome message.
- Power off.
- `print!`, `println!`, `eprint`, and `eprintln!` macros.
- Test suite.

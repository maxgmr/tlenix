# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Planned]

- Custom global allocator.
- Environment variables.
- `mash` prompt should react to exit code of last-executed process.
- `mash` ability to execute programs.
- Fun BSOD-style screen on panic.

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

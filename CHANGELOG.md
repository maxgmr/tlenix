# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Planned]

- Custom global allocator.
- Environment variables.
- Trait describing how various types can be converted into syscall args.
- Trait describing how various types can be converted from syscall outputs.

## [0.1.0-alpha.7] - IN PROGRESS

### Changed

- General implementation overhaul/reorganization.

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

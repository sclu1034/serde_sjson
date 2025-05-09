# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [1.2.1] - 2025-04-21

### Changed

- update [nom](https://crates.io/crates/nom) to v8

## [1.2.0] - 2024-03-21

### Added

- publishing to [crates.io](https://crates.io)

## [v1.1.0] - 2024-03-21

### Added

- implement serializing into generic `io::Write`

### Fixed

- fix parsing CRLF

## [v1.0.0] - 2023-03-10

### Added

- implement literal strings

### Fixed

- fix serializing strings containing `:`
- fix serializing certain escaped characters

## [v0.2.4] - 2023-03-01

### Fixed

- fix incorrect parsing of unquoted strings

## [v0.2.3] - 2023-02-24

### Fixed

- support backslashes in delimited strings

## [v0.2.2] - 2023-02-18

### Fixed

- fix deserialization failing on arrays and objects in some cases

## [v0.2.1] - 2022-12-28

### Fixed

- fix serializing Unicode

## [v0.2.0] - 2022-11-25

### Added

* parsing & deserialization

## [v0.1.0] - 2022-11-18

### Added

* initial release
* serialization

# Changelog
All notable changes to this project will be documented in this file.

## [0.4.0] - 2025-09-21
### Added

- Implement iterator over contacts: filters by tag/domain.
- chainable filters in tests (iter().filter(...).take(...)).
- Add read-only views returning &Contact slices (no clones), with compile‑time borrow checks pass; zero-copy listings.
- Add black‑box tests in tests/cli.rs (spawn binary, assert output).
- Introduce micro‑benchmarking using criterion.

---

## [0.3.0] - 2025-08-30
### Added
- Persist `Store::file`.
- Implemented `MemStore` and `FileStore` with traits.
- Added custom errors (`enum AppError`) and map I/O/parse errors to `AppError`.
- Added Generic helper to standardize input loop.
- Introduced Github Actions.

### Changed
- Fixed validation from Regex.

---

## [0.2.0] - 2025-08-22
### Added
- Split project into modules: `cli`, `domain`, `validation`, and `store::mem`.
- Added **unit tests** for validation functions.
- Added **unit tests** for CLI command parsing (via `parse_command`).
- Added **integration tests** under `tests/validation.rs`.
- Introduced library crate (`src/lib.rs`) alongside binary target.

### Changed
- Refactored `main.rs` to delegate logic to `cli::run_cli`.
- Moved data structure definition (`Contact`) into `domain.rs`.

### Fixed
- Improved error handling for invalid user input.
- More robust validation checks for name, phone, and email.

---

## [0.1.0] - 2025-08-20
### Added
- Initial implementation of **Contact Manager** CLI.
- Features:
  - Add a contact
  - List all contacts
  - Delete a contact
  - Exit program
- Basic input validation for name, phone, and email.
- In-memory contact store.



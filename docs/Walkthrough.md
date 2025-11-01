# Week 7

## Tasks
- Add ```sync``` commands to merge from a file (three-way merge policy: keep, overwrite, duplicate handling)..

- Document tmux workflow on mobile (Termux): install Rust, run CLI, sync via local files.

## ```sync``` command
This feature covers the scenario of merging another JSON contact list to the existing JSON contacts. With a provided policy, duplicate contacts can be either kept, overwritten or duplicated.

### Command
```rust
cargo run -- sync "another_contacts.json" --policy keep
```
```rust
cargo run -- sync "another_contacts.json" --policy overwrite
```
```rust
cargo run -- sync "another_contacts.json" --policy duplicate
```

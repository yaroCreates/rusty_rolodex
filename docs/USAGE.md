# Rolodex CLI – Usage Guide





## Add a Contact

```bash
cargo run -- add --name "Alice" --phone "12345" --email "alice@example.com"
```

### Output
```bash
✅ Added contact: Alice (alice@example.com)
```
## List Contacts

```bash
cargo run -- list
```
### Sort by name
```bash
cargo run -- list --sort name
```

### Sort by email
```bash
cargo run -- list --sort email
```

## Delete a Contact
```bash
cargo run -- delete --name "Alice"
```

### Output
```bash
🗑️ Removed contact: Alice
```
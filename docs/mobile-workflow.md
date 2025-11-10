# 📱 Termux + tmux Workflow for Rust CLI
## 🥅 Goal
Run, manage, and sync the rolodex CLI tool entirely on your Android phone — using Termux + tmux, with JSON files.

## Steps
### 1. Install Termux & Required Packages


Termux is a Linux-like environment for Android. <br/> You can get it safely from Google store or other reliable stores.

Once installed, open Termux and run:
```bash
pkg update && pkg upgrade
pkg install git clang curl tmux
```

### 2. Install Rust
Rust runs natively inside Termux. Install via `rustup`:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Then restart your shell:
```bash
source $HOME/.cargo/env
```

Check that Rust works:
```bash
rustc --version
cargo --version

```

### 3. Clone the CLI Project
```bash
git clone https://github.com/yaroCreates/rusty_rolodex.git
cd rusty_rolodex
```

### 4. Sync Contact via local files
Give Termux permission to access storage:

```bash
termux-setup-storage
```
Grant Termux the necessary access to your files, in this case to your file directory.

Assumning your file is in the `/Download` directory, you can confirm by running:

```bash
ls ~/downloads
```

After confirming, you can perform the `Sync` by running:

```bash
cargo run -- sync --file ~/downloads/contacts.json --policy "keep | overwrite | duplicate"
```

### Typical Workflow Example

| | Action | Command
|--- |-----|----|
| 1.| Add new contact | `cargo run -- add --name "Jane Doe" --email "jane@mail.com" --phone "555-0101"` |
|2. | List contacts | `cargo run -- list` |
|3. | Sync from local file | `cargo run -- sync --file location --policy keep` |


# Week 9

# HTTP Import/Export (Sync)

## Tasks
- Add HTTP capabilities so the CLI can push its contacts to a real remote service and pull them back later.
    - Use a blocking HTTP client (`ureq` or `reqwest` in blocking mode).
    - Target a real JSON-capable endpoint.


## Breakdown

The remote service I used is *`JSONBIN`*. The reason for my choice is because of the ease in saving and retrieving JSON data. Nothing more nothing less. Also, I used the `reqwest` Http client library.

---

### Import
```bash
cargo run -- import --from "https://api.jsonbin.io/v3/b/6914b5b043b1c97be9a93fc5"

```
A bin can be manually created and populated with JSON contacts. The bin can be imported into the CLI app using the command above.

---

### Export

```bash
cargo run -- export --to "https://api.jsonbin.io/v3/b"
```
During exporting, JSONBIN creates a bin and stores your JSON. All you need to provide is the api's base url ( In this case `https://api.jsonbin.io/v3/b`).

---


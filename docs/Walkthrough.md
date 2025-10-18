# Week 6

## Tasks
- Implement simple in-memory index by name/email domain.

- Add fuzzy contains search (case-insensitive).

- Optional: use Rust std threads (not Rayon) to perform searches concurrently across chunks of data.

## Thought on `Searching & Indexing`
Before really starting on the week's tasks, I had to study about Indexing in Database and how it is works. Database indexing is a technique that uses a data structure, like a sorted list, to speed up data retrieval from a database table. It works by creating a special lookup table that holds indexed column values and pointers to the original values, allowing the database to find data much faster than scanning the entire table.

---
With this understanding, I made use of the `Hash Maps` to achieve this, storing and grouping the position of the values first before retrieving them.

```rust
pub struct ContactsIndex {
    name_map: HashMap<String, Vec<usize>>,
    domain_map: HashMap<String, Vec<usize>>,
}
```

### Implement simple in-memory index by name/email domain.

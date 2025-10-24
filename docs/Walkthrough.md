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

## Benchmarking
###  Name lookup


| **Name Lookup** | **10k** | **50k** | **100K** |
|:-------------|:----------:|--------:|:---------|
| Citerion     | 39.144 ns  | 39.655 ns | 39.522 ns 
| std::time    | 2.661 µs   | 2.498µs  | 2.442µs

### Domain Lookup
| **Email Lookup** | **10k** | **50k** | **100K** |
|:-------------|:----------:|--------:|:---------|
| Citerion     | 41.665 ns         | 41.049 ns  | 42.956 ns
| std::time    | 27.554 µs         | 152.51µs  | 280.362µs


### Fuzzy Search
| **Fuzzy Search** | **10k** | **50k** | **100K** |
|:-------------|:----------:|--------:|:---------|
| Citerion     | 23.708 ms         | 115.46 ms  | 241.22 ms
| std::time    | 625.84 ms         | 3.33 s  | 6.76 s


### Fuzzy Search with Concurrency
| **Fuzzy Search(C)** | **10k** | **50k** | **100K** |
|:-------------|:----------:|-----------:|:---------|
| Citerion     | 10.691 ms         | 53.167 ms  | 95.945 ms
| std::time    | 133.36 ms         | 673.19 ms  | 1.37 s

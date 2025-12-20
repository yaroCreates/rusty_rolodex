# Performance Benchmarks

Benchmarks were executed locally to measure:
- Contact construction
- HashMap insertion
- Index creation (name & domain lookup)


---

## Benchmark Environment

| Component        | Value |
|------------------|------|
| Rust Version     | stable |
| Build Profile    | `--release` |
| Benchmark Tool   | Criterion `0.5` |
| Data Structure  | `HashMap<Uuid, Contact>` |
| UUID Strategy   | `Uuid::new_v4()` |
| Time Source     | `chrono::Utc::now()` |

---

## Benchmarked Operation

**Operation:** Build `N` contacts and index them

Steps:
1. Generate `N` contacts
2. Insert into a pre-allocated `HashMap`
3. Build secondary indexes:
   - name → `Vec<Uuid>`
   - email domain → `Vec<Uuid>`

---

## Results Summary

### Contact Build + Index Time

| Dataset Size | Build Contacts | Build Indexes | Name lookup | Domain lookup | Fuzzy search | Fuzzy concurrent search |
|-------------:|------------:|--------:|-----------:|----:|-----:|-----:|
| 1,000        | 1.9 ms    | 999.5 us | 115.5 ns | 123.8 ns | 9.8 ms | 11.8 ms |
| 5,000        | 10.1 ms   | 4.9 ms   | 115.9 ns | 125.9 ns | 40.8 ms | 53 ms
| 10,000       | 19.5 ms   | 10.5 ms  | 115.1 ns | 121.4 ns | 115 ms | 101.9ms 
| 50,000       | 127.4 ms  | 81.2 ms  | 115.9 ns | 123.4ns  | 366.9 ms | 358.4 ms

---


## Observations

---

## Running the Benchmarks

```bash
cargo bench

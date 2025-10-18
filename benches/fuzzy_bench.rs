use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use rusty_rolodex::domain::{Contact, ContactsIndex};

fn bench_fuzzy_search(c: &mut Criterion) {
    let contacts: Vec<_> = (0..10_000)
        .map(|i| Contact {
            name: format!("Person{}", i),
            email: format!("person{}@mail.com", i),
            phone: format!("232323323211"),
            tags: vec!["bench".into()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .collect();

    let index = ContactsIndex::build(&contacts);

    c.bench_function("fuzzy_search_concurrent", |b| {
        b.iter(|| {
            let _ = index.fuzzy_search_concurrency("Person1234", &contacts, 4);
        })
    });
}

criterion_group!(benches, bench_fuzzy_search);
criterion_main!(benches);

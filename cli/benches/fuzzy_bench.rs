use std::collections::HashMap;

use chrono::Utc;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rolodex_core::domain::{Contact, ContactsIndex};
use uuid::Uuid;

fn build_contacts(count: usize) -> HashMap<Uuid, Contact> {
    let mut map = HashMap::with_capacity(count);

    for i in 0..count {
        let id = Uuid::new_v4();
        let now = Utc::now();

        map.insert(
            id,
            Contact {
                id,
                name: format!("Person{}", i),
                email: format!("person{}@mail.com", i),
                phone: vec![format!("232323323211")],
                tags: vec!["bench".into()],
                created_at: now,
                updated_at: now,
            },
        );
    }
    map
}

fn bench_fuzzy_search(c: &mut Criterion) {
    let contacts = build_contacts(50_000);

    let index = ContactsIndex::build(&contacts);

    c.bench_function("build 50k contacts", |b| {
        b.iter(|| {
            black_box(build_contacts(50_000));
        })
    });

    c.bench_function("build indexes", |b| {
        b.iter(|| {
            black_box(ContactsIndex::build(&contacts));
        })
    });

    c.bench_function("name_lookup", |b| {
        b.iter(|| {
            // let index = build_contacts(10_000);
            let _ = index.lookup_name("mail.com");
        })
    });

    c.bench_function("domain_lookup", |b| {
        b.iter(|| {
            let _ = index.lookup_domain("Person1234");
        })
    });

    c.bench_function("fuzzy_search", |b| {
        b.iter(|| {
            let _ = index.fuzzy_search("Person1234", &contacts, 2);
        })
    });

    c.bench_function("fuzzy_search_concurrent", |b| {
        b.iter(|| {
            let _ = index.fuzzy_search_concurrency("Person1234", &contacts, 2);
        })
    });
}

criterion_group!(benches, bench_fuzzy_search);
criterion_main!(benches);

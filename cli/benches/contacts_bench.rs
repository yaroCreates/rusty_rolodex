use std::collections::HashMap;

use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use rolodex_core::domain::{Contact, Contacts};
use uuid::Uuid;

fn sample_contacts(n: usize) -> Contacts {
    let mut items = HashMap::with_capacity(n);

    for i in 0..n {
        let id = Uuid::new_v4();
        let now = Utc::now();

        items.insert(
            id,
            Contact {
                id,
                name: format!("User{}", i),
                phone: vec![format!("232323323211")],
                email: format!("user{}@example.com", i),
                tags: vec!["work".into()],
                created_at: now,
                updated_at: now,
            },
        );
    }

    Contacts::new(items)
}

fn bench_filtering(c: &mut Criterion) {
    let contacts = sample_contacts(10_000);

    c.bench_function("filter_by_tag_work", |b| {
        b.iter(|| {
            let count = contacts.iter().filter(|c| c.has_tag("work")).count();
            assert_eq!(count, 10_000);
        })
    });

    c.bench_function("filter_by_domain_example.com", |b| {
        b.iter(|| {
            let count = contacts
                .iter()
                .filter(|c| c.has_domain("example.com"))
                .count();
            assert_eq!(count, 10_000);
        })
    });
}

criterion_group!(benches, bench_filtering);
criterion_main!(benches);

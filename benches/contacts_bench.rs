use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use rusty_rolodex::domain::{Contact, Contacts};

fn sample_contacts(n: usize) -> Contacts {
    let items = (0..n)
        .map(|i| {
            Contact::new(
                &format!("User{}", i),
                &format!("000{}", i),
                &format!("user{}@example.com", i),
                vec!["work".into()],
                Utc::now(),
                Utc::now()
            )
        })
        .collect();
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

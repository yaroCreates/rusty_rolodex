use criterion::{Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct Contacts {
    items: Vec<Contact>,
}

impl Contacts {
    pub fn new(items: Vec<Contact>) -> Self {
        Self { items }
    }

    pub fn iter(&'_ self) -> ContactsIter<'_> {
        ContactsIter {
            inner: self.items.iter(),
        }
    }
}

impl Contact {
    pub fn new(name: &str, phone: &str, email: &str, tags: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            phone: phone.to_string(),
            email: email.to_string(),
            tags,
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn has_domain(&self, domain: &str) -> bool {
        self.email.ends_with(&format!("@{}", domain))
    }
}

pub struct ContactsIter<'a> {
    inner: std::slice::Iter<'a, Contact>,
}

impl<'a> Iterator for ContactsIter<'a> {
    type Item = &'a Contact;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn sample_contacts(n: usize) -> Contacts {
    let items = (0..n)
        .map(|i| {
            Contact::new(
                &format!("User{}", i),
                &format!("000{}", i),
                &format!("user{}@example.com", i),
                vec!["work".into()],
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

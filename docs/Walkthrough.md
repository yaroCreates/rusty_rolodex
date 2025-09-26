# Walkthrough (Week 2)


The Contact app has been updated to either run on `Memory storage` and `File storage`, with the latter persisted.

By Default, the application runs on the File Storage (`cargo run`).

![alt text](image.png)

With the use of .env variable, the runtime store can be changed to `MemStore` by running:.

`STORE_TYPE=mem cargo run`

![alt text](image-1.png)

### MemStore
Contacts added to the `MemStore` are lost after the application is ended or restarted.

![alt text](image-2.png)

Adding Contact

![alt text](image-4.png)

Viewing contact list

![alt text](image-5.png)

After restarting, the Contact list is cleared...

![alt text](image-6.png)
---

### FileStore
`cargo run` lunches the app on the default store (FileStore)

![alt text](image-7.png)

View Contact list...

![alt text](image-8.png)

Let's add a contact to test the persistence:

![alt text](image-9.png)

Resetting the app

![alt text](image-10.png)

![alt text](image-11.png)

---
# Week 4

## Correction from week 3
- ✅️ Pipeline fix
- ✅️ Check for duplicate name before adding contacts
- ✅️ Deleting contact with same name (Going to be concluded)
- ✅️ Updated Phone number validation function name
- ✅️ Contact.txt -> Contact.json Migration
- ✅️ Versioned release via Git tag

## Tasks
Iterator over contact - filter with tags and domain.

In achieving filters by tags or domain, the Contact Struct had to be first updated to accept tags upon creation. In order to support backward compatiblity, the tag field is defined optional so as to support contacts that were created before the feature.

```rust
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
```

Running the command to return the contact...

```bash
cargo run -- list
```
![alt text](media/image-12.png)

Running filter by domain
```bash
cargo run -- list --domain "example.com"
```
![alt text](image-12.png)

## Integration test (Black-box testing)
The integration test contains a full run through of the entire app, creating two scenarios for:
- Adding and Listing
- Adding, Deleting and Listing

![alt text](media/image-13.png)

## Demo
![alt text](media/week-4-demo.gif)

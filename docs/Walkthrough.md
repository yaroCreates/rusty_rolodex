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

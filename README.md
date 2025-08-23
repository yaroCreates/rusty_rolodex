# 📇 Contact Manager CLI (Rust)

A simple command-line contact manager written in Rust.  
Supports adding, viewing, deleting contacts, saving to a JSON file, input validation, and timestamping.

---

## ✨ Features

- Add new contacts with name, phone, email, and created date.
- View all contacts sorted alphabetically.
- Delete contacts by name.
- Input validation:
  - Name: Alphabetic characters only.
  - Phone: Digits only, at least 10 digits.
  - Email: Valid email format using regex.

---

## 🛠 Installation

1. Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

2. Clone this repository:

   ```bash
   git clone https://github.com/your-username/contact-manager-cli.git
   cd contact-manager-cli
   ```
3. Build and run:
    ```bash
    cargo run
    ```

## 📋Usage

When you run the program, you will see a menu:

1. Add a contact
2. View all contacts
3. Delete contact by name
4. Exit

- Add a contact: Enter name, phone number, and email. Data will be validated.

- View all contacts: Lists contacts alphabetically by name.

- Delete a contact: Enter the name of the contact to delete.

- Exit




## 🔧 Dependencies

This project uses the following crates:

- regex — for phone and email validation.

Add them in Cargo.toml


## 👨‍💻 Author
- yaroCreates 



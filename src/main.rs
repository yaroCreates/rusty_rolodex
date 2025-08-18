use std::io::{self, Write};

struct Contact {
    name: String,
    phone: String,
    email: String,
}

fn main() {
    let mut contacts = Vec::new();
    contacts.push(Contact {
        name: "James Yaro".to_string(),
        phone: "08122121474".to_string(),
        email: "onuhjamesyaro@gmail.com".to_string(),
    });

    loop {
        println!("\n");
        println!("Welcome to my Contact Manager");
        println!("\n");
        println!("1. Add a contact");
        println!("2. List all contacts");
        println!("3. Exit");
    
        io::stdout().flush().unwrap();
        let menu_option = read_input();

        match menu_option.as_str() {
            "1" => {
                add_contact(&mut contacts);
                // save_contacts(&contacts);
            }
            "2" => view_contacts(&mut contacts),
            "3" => {
                println!("Hope to see you back soon!");
                break;
            }
            _ => println!("Invalid selection, please choose from 1-3."),
        }
    
        

    }

}

fn add_contact(contacts: &mut Vec<Contact>) {
    println!("\n--- Add Contact ---");
    
    let name = loop {
        print!("Enter name: ");
        io::stdout().flush().unwrap();
        let input = read_input();
        break input;
        
    };

    let phone = loop {
        print!("Enter phone: ");
        io::stdout().flush().unwrap();
        let input = read_input();
        break input;
        
    };

    let email = loop {
        print!("Enter email: ");
        io::stdout().flush().unwrap();
        let input = read_input();
        break input;
    };

    contacts.push(Contact {
        name,
        phone,
        email,
    });
}

fn view_contacts(contacts: &mut Vec<Contact>) {
    for (i, c) in contacts.iter().enumerate() {
        println!(
            "{}. Name: {}, Phone: {}, Email: {}",
            i + 1,
            c.name,
            c.phone,
            c.email
        )
    }
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

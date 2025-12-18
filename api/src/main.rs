use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{Json, Router, extract::State, routing::get};
use chrono::{DateTime, Utc};
// use rusty_rolodex::{core::domain::AppState, domain::Contact, prelude::AppError};
use rolodex_core::{
    domain::Contact,
    error::AppError,
    store::{ContactStore, FileStore},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialContact {
    pub name: String,
    pub phone: Vec<String>,
    pub email: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
    pub id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // let shared_state = Arc::new(Mutex::new(AppState::new("another_contacts.json")));
    let store = Arc::new(Mutex::new(FileStore::new("contacts.json")));
    // let store=Arc::new(FileStore::new("contacts.json"));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/contacts", get(get_contacts).post(post_contacts))
        .with_state(store);

    async fn get_contacts(State(state): State<Arc<Mutex<FileStore>>>) -> Json<Vec<Contact>> {
        let guard = state.lock().unwrap();
        let imported_contacts = guard.load().expect("Panic happened!");

        let data = imported_contacts.into_values().collect();

        Json(data)
    }

    // async fn get_all(store: axum::extract::State<Arc<FileStore>>) -> Json<Vec<Contact>> {
    //     let data: Vec<Contact> = store.load().unwrap().into_values().collect();
    //     Json(data)
    // }

    async fn post_contacts(
        State(state): State<Arc<Mutex<FileStore>>>,
        // store: axum::extract::State<Arc<FileStore>>,
        Json(mut payload): Json<Vec<Contact>>,
    ) -> Json<Vec<Contact>> {
        println!("Axum payload:{:?}", payload.clone());

        let guard = state.lock().unwrap();

        let mut new_contacts: HashMap<Uuid, Contact> = HashMap::new();

        for contact in payload.iter_mut() {
            let uu_id = Uuid::new_v4();
            new_contacts.insert(
                uu_id,
                Contact {
                    id: uu_id,
                    name: contact.name.clone(),
                    phone: contact.phone.clone(),
                    email: contact.email.clone(),
                    tags: contact.tags.clone(),
                    created_at: contact.created_at,
                    updated_at: contact.updated_at,
                },
            );
        }

        guard.save(new_contacts).expect("Panic occurred!");

        println!("Axum payload after inserting:{:?}", payload);
        Json(payload)
    }

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

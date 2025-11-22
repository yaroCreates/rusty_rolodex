use std::sync::{Arc, Mutex};

use axum::{Json, Router, extract::State, routing::get};
use chrono::{DateTime, Utc};
use rusty_rolodex::{core::domain::AppState, domain::Contact, prelude::AppError};

use serde::{Deserialize, Serialize};

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
    let shared_state = Arc::new(Mutex::new(AppState::new("another_contacts.json")));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/contacts", get(get_contacts).post(post_contacts))
        .with_state(shared_state);

    async fn get_contacts(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<Contact>> {
        // let data = fs::read_to_string("another_contacts.json").expect("Path wrong");

        // let imported_contacts: Vec<Contact> = serde_json::from_str(&data)
        //     .map_err(|e| AppError::Parse(format!("Error, JSON... : {}", e))).expect("Issues getting contacts");

        let guard = state.lock().unwrap();
        let imported_contacts = guard.load().expect("Panic happened!");

        Json(imported_contacts)
    }

    async fn post_contacts(
        State(state): State<Arc<Mutex<AppState>>>,
        Json(mut payload): Json<Vec<SpecialContact>>,
    ) -> Json<Vec<SpecialContact>> {
        println!("Axum payload:{:?}", payload.clone());

        let guard = state.lock().unwrap();

        let mut new_contacts: Vec<Contact> = Vec::new();

        for contact in payload.iter_mut() {
            contact.id = Some(uuid::Uuid::new_v4().to_string());

            new_contacts.push(Contact {
                name: contact.name.clone(),
                phone: contact.phone.clone(),
                email: contact.email.clone(),
                tags: contact.tags.clone(),
                created_at: contact.created_at,
                updated_at: contact.updated_at,
            });
        }

        guard.save(&new_contacts).expect("Panic occurred!");

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

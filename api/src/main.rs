use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    status: String,
    message: String,
    data: Option<Vec<Contact>>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let store = Arc::new(Mutex::new(FileStore::new("contacts.json")));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/contacts", get(get_contacts).post(post_contacts))
        .route(
            "/contacts/{contact_id}",
            put(edit_contact).delete(delete_contact),
        )
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
        Json(mut payload): Json<Vec<Contact>>,
    ) -> Json<ApiResponse> {
        println!("Axum payload:{:?}", payload.clone());

        let guard = state.lock().unwrap();

        //Get the current contacts
        let mut contacts = guard.load().expect("Panic happened when fetching contacts");

        for contact in payload.iter_mut() {
            let uu_id = Uuid::new_v4();
            contacts.insert(
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
        guard.save(contacts).expect("Panic occurred!");

        println!("Axum payload after inserting:{:?}", payload);

        let api_response = ApiResponse {
            status: "success".to_string(),
            message: "Contact created successfully".to_string(),
            data: Some(payload),
        };

        Json(api_response)
    }

    async fn delete_contact(
        State(state): State<Arc<Mutex<FileStore>>>,
        Path(contact_id): Path<Uuid>,
    ) -> Json<ApiResponse> {
        //
        let guard = state.lock().unwrap();

        //Get the contacts
        let mut contacts = guard.load().expect("Error occurred while getting contacts");

        let result = contacts.remove(&contact_id);

        let api_response: ApiResponse;

        if let Some(response) = result {
            guard.save(contacts).expect("Panic occured");

            api_response = ApiResponse {
                status: "success".to_string(),
                message: format!("Contact with id:{} deleted!", contact_id),
                data: Some(vec![response]),
            };
            Json(api_response)
        } else {
            api_response = ApiResponse {
                status: "error".to_string(),
                message: format!("Contact with id: {} not found", contact_id),
                data: None,
            };
            Json(api_response)
        }
    }

    async fn edit_contact(
        State(state): State<Arc<Mutex<FileStore>>>,
        Path(contact_id): Path<Uuid>,
        Json(payload): Json<Contact>,
    ) -> Json<ApiResponse> {
        let guard = state.lock().unwrap();

        //Get the contacts
        let mut contacts = guard.load().expect("Error occurred");

        //Get the contact by ID
        let contact = contacts.get(&contact_id).cloned();

        let api_response: ApiResponse;

        if let Some(mut data) = contact {
            if !payload.name.is_empty() {
                data.name = payload.name;
            }
            if !payload.email.is_empty() {
                data.email = payload.email;
            }
            if !payload.phone.is_empty() {
                data.phone = payload.phone;
            }
            data.updated_at = Utc::now();

            contacts.insert(data.id, data.clone());

            guard.save(contacts).expect("Panic occurred");

            api_response = ApiResponse {
                status: "success".to_string(),
                message: format!("Contact with id:{} updated!", contact_id),
                data: Some(vec![data]),
            };
            Json(api_response)
        } else {
            api_response = ApiResponse {
                status: "error".to_string(),
                message: format!("Contact with id: {} not found", contact_id),
                data: None,
            };
            Json(api_response)
        }
    }

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// File: src/main.rs

mod storage;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use storage::{DbStorage, Item};

// Type alias untuk Shared State lintas thread
type AppState = Arc<Mutex<DbStorage>>;

// Payload DTO (Data Transfer Object)
#[derive(Deserialize)]
struct CreateItemPayload {
    name: String,
    price: f64,
    stock: u32,
}

#[derive(Deserialize)]
struct UpdateItemPayload {
    price: Option<f64>,
    stock: Option<u32>,
}

#[tokio::main]
async fn main() {
    let db = DbStorage::new("inventory.redb").expect("Gagal inisialisasi Database");
    let state: AppState = Arc::new(Mutex::new(db));

    let app = Router::new()
        .route("/items", get(get_items).post(create_item))
        .route("/items/:id", put(update_item).delete(delete_item))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server REST API berjalan di http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

// 1. GET /items
async fn get_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, (StatusCode, String)> {
    let db = state.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let items = db.list_items().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

// 2. POST /items
async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemPayload>,
) -> Result<(StatusCode, Json<Item>), (StatusCode, String)> {
    let db = state.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let item = db
        .add_item(payload.name, payload.price, payload.stock)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(item)))
}

// 3. PUT /items/:id
async fn update_item(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateItemPayload>,
) -> Result<Json<Item>, (StatusCode, String)> {
    let db = state.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match db.update_item(id, payload.price, payload.stock) {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err((StatusCode::NOT_FOUND, format!("ID {} tidak ditemukan", id))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// 4. DELETE /items/:id
async fn delete_item(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match db.delete_item(id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((StatusCode::NOT_FOUND, format!("ID {} tidak ditemukan", id))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
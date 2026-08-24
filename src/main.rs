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
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

type AppState = Arc<Mutex<DbStorage>>;

#[derive(Deserialize, ToSchema)]
struct CreateItemPayload {
    #[schema(example = "SSD NVMe 1TB")]
    name: String,
    #[schema(example = 1250000.0)]
    price: f64,
    #[schema(example = 10)]
    stock: u32,
}

#[derive(Deserialize, ToSchema)]
struct UpdateItemPayload {
    #[schema(example = 1150000.0)]
    price: Option<f64>,
    #[schema(example = 8)]
    stock: Option<u32>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_items,
        create_item,
        update_item,
        delete_item
    ),
    components(
        schemas(Item, CreateItemPayload, UpdateItemPayload)
    ),
    tags(
        (name = "Inventory", description = "Endpoint manajemen inventaris barang")
    ),
    info(
        title = "Inventory REST API",
        version = "1.0.0",
        description = "Dokumentasi REST API Inventaris dengan Axum dan Redb"
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let db = DbStorage::new("inventory.redb").expect("Gagal inisialisasi Database");
    let state: AppState = Arc::new(Mutex::new(db));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/items", get(get_items).post(create_item))
        .route("/items/:id", put(update_item).delete(delete_item))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server REST API berjalan di http://127.0.0.1:3000");
    println!("Swagger UI aktif di http://127.0.0.1:3000/swagger-ui");

    axum::serve(listener, app).await.unwrap();
}

/// Ambil semua data barang
#[utoipa::path(
    get,
    path = "/items",
    tag = "Inventory",
    responses(
        (status = 200, description = "Daftar barang berhasil diambil", body = Vec<Item>),
        (status = 500, description = "Internal Server Error", body = String)
    )
)]
async fn get_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, (StatusCode, String)> {
    let db = state.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let items = db.list_items().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

/// Tambah barang baru
#[utoipa::path(
    post,
    path = "/items",
    tag = "Inventory",
    request_body = CreateItemPayload,
    responses(
        (status = 201, description = "Barang berhasil dibuat", body = Item),
        (status = 500, description = "Internal Server Error", body = String)
    )
)]
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

/// Perbarui barang berdasarkan ID
#[utoipa::path(
    put,
    path = "/items/{id}",
    tag = "Inventory",
    params(
        ("id" = u32, Path, description = "ID Barang yang akan diupdate")
    ),
    request_body = UpdateItemPayload,
    responses(
        (status = 200, description = "Barang berhasil diupdate", body = Item),
        (status = 404, description = "Barang tidak ditemukan", body = String),
        (status = 500, description = "Internal Server Error", body = String)
    )
)]
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

/// Hapus barang berdasarkan ID
#[utoipa::path(
    delete,
    path = "/items/{id}",
    tag = "Inventory",
    params(
        ("id" = u32, Path, description = "ID Barang yang akan dihapus")
    ),
    responses(
        (status = 204, description = "Barang berhasil dihapus"),
        (status = 404, description = "Barang tidak ditemukan", body = String),
        (status = 500, description = "Internal Server Error", body = String)
    )
)]
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
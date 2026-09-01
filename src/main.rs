// File: src/main.rs

mod storage;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State, DefaultBodyLimit, Multipart,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use storage::{DbStorage, Item};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

// 1. Tambahkan channel Broadcast ke dalam State
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<DbStorage>>,
    tx: broadcast::Sender<String>,
}

#[derive(Deserialize, ToSchema)]
struct CreateItemPayload {
    name: String,
    price: f64,
    stock: u32,
}

#[derive(Deserialize, ToSchema)]
struct UpdateItemPayload {
    price: Option<f64>,
    stock: Option<u32>,
}

#[derive(OpenApi)]
#[openapi(
    paths(get_items, create_item, update_item, delete_item),
    components(schemas(Item, CreateItemPayload, UpdateItemPayload)),
    tags((name = "Inventory", description = "Manajemen Inventaris Real-Time"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let db = DbStorage::new("inventory.redb").expect("Gagal inisialisasi Database");
    
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/items", get(get_items).post(create_item))
        .route("/items/:id", put(update_item).delete(delete_item))
        .route("/scan", axum::routing::post(scan_item)) // Panggilan ke fungsi di bawah
        .route("/ws", get(ws_handler))
        .layer(cors)
        // Batas maksimal 20 MB untuk gambar resolusi tinggi
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024)) 
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await.unwrap();
    println!("Server REST API & WS berjalan di http://127.0.0.1:3030");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}


async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break; // Jika browser ditutup, hentikan loop
        }
    }
}

#[utoipa::path(get, path = "/items", tag = "Inventory", responses((status = 200, body = Vec<Item>)))]
async fn get_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, (StatusCode, String)> {
    let db = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let items = db.list_items().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

#[utoipa::path(post, path = "/items", tag = "Inventory", request_body = CreateItemPayload, responses((status = 201, body = Item)))]
async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItemPayload>,
) -> Result<(StatusCode, Json<Item>), (StatusCode, String)> {
    let db = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let item = db.add_item(payload.name, payload.price, payload.stock)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Broadcast sinyal REFRESH ke semua client via WebSocket
    let _ = state.tx.send("REFRESH".to_string());
    
    Ok((StatusCode::CREATED, Json(item)))
}

#[utoipa::path(put, path = "/items/{id}", tag = "Inventory", params(("id" = u32, Path)), request_body = UpdateItemPayload, responses((status = 200, body = Item)))]
async fn update_item(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateItemPayload>,
) -> Result<Json<Item>, (StatusCode, String)> {
    let db = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match db.update_item(id, payload.price, payload.stock) {
        Ok(Some(item)) => {
            let _ = state.tx.send("REFRESH".to_string());
            Ok(Json(item))
        },
        Ok(None) => Err((StatusCode::NOT_FOUND, format!("ID {} tidak ditemukan", id))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[utoipa::path(delete, path = "/items/{id}", tag = "Inventory", params(("id" = u32, Path)), responses((status = 204)))]
async fn delete_item(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match db.delete_item(id) {
        Ok(true) => {
            let _ = state.tx.send("REFRESH".to_string());
            Ok(StatusCode::NO_CONTENT)
        },
        Ok(false) => Err((StatusCode::NOT_FOUND, format!("ID {} tidak ditemukan", id))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn scan_item(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Item>), (StatusCode, String)> {
    
    // 1. Ekstrak file gambar dari request Multipart frontend
    let mut image_bytes = Vec::new();
    let mut file_name = String::new();
    
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            file_name = field.file_name().unwrap_or("image.jpg").to_string();
            let data = field.bytes().await.map_err(|e| {
                (StatusCode::BAD_REQUEST, format!("Gagal membaca file: {}", e))
            })?;
            image_bytes = data.to_vec();
            break;
        }
    }

    if image_bytes.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "File gambar tidak ditemukan".to_string()));
    }

    // 2. Siapkan HTTP Client (reqwest) untuk memanggil API Python (FastAPI)
    let client = reqwest::Client::new();
    
    // Bungkus bytes menjadi format multipart untuk dikirim ke Python
    let part = reqwest::multipart::Part::bytes(image_bytes)
        .file_name(file_name)
        .mime_str("image/jpeg")
        .unwrap();
    let form = reqwest::multipart::Form::new().part("file", part);

    // Kirim ke service Python lokal yang berjalan di port 8000
    let ocr_response = client
        .post("http://127.0.0.1:8000/process-image")
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            (StatusCode::SERVICE_UNAVAILABLE, format!("Service Python OCR mati: {}", e))
        })?;

    // Asumsi: Python mengembalikan JSON { "extracted_text": "KODE-BARANG-123" }
    #[derive(serde::Deserialize)]
    struct OcrResult {
        extracted_text: String,
    }
    
    let ocr_data: OcrResult = ocr_response.json().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Format respon Python salah: {}", e))
    })?;

    // 3. Simpan nama barang (hasil OCR) ke database Redb 
    let db = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let new_item = db.add_item(ocr_data.extracted_text, 0.0, 1)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 4. Trigger WebSocket agar frontend langsung Refresh
    let _ = state.tx.send("REFRESH".to_string());

    Ok((StatusCode::CREATED, Json(new_item)))
}
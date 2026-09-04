mod storage;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use storage::{DbStorage, Item};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

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
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state)
        .fallback(static_handler);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await.unwrap();
    println!("Server Single-Binary berjalan di http://127.0.0.1:3030");
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(index) = Assets::get("index.html") {
                ([(header::CONTENT_TYPE, "text/html")], index.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
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
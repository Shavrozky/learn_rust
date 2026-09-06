use gloo_net::http::Request;
use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlAnchorElement, MessageEvent, WebSocket};

const API_BASE: &str = "/items";
const LOGS_ENDPOINT: &str = "/api/logs";

fn get_ws_url() -> String {
    if let Some(window) = web_sys::window() {
        let location = window.location();
        let host = location.host().unwrap_or_else(|_| "127.0.0.1:3030".to_string());
        let proto = match location.protocol().as_deref() {
            Ok("https:") => "wss:",
            _ => "ws:",
        };
        format!("{}//{}/ws", proto, host)
    } else {
        "ws://127.0.0.1:3030/ws".to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub stock: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: u64,
    pub item_id: u32,
    pub change_type: String,
    pub delta: i32,
    pub timestamp: u64,
}

#[derive(Serialize)]
struct CreateItemPayload {
    name: String,
    price: f64,
    stock: u32,
}

#[derive(Serialize)]
struct UpdateItemPayload {
    price: Option<f64>,
    stock: Option<u32>,
}

#[component]
fn App() -> impl IntoView {
    let (trigger, set_trigger) = create_signal(0);
    let (search_query, set_search_query) = create_signal(String::new());

    let ws_url = get_ws_url();
    if let Ok(ws) = WebSocket::new(&ws_url) {
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                if txt == "REFRESH" {
                    set_trigger.update(|n| *n += 1);
                }
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    // Resource Data Barang
    let items_resource = create_resource(
        move || trigger.get(),
        |_| async move {
            Request::get(API_BASE)
                .send()
                .await
                .ok()?
                .json::<Vec<Item>>()
                .await
                .ok()
        },
    );

    // Resource Data Log Transaksi
    let logs_resource = create_resource(
        move || trigger.get(),
        |_| async move {
            Request::get(LOGS_ENDPOINT)
                .send()
                .await
                .ok()?
                .json::<Vec<AuditLog>>()
                .await
                .ok()
        },
    );

    let name_ref = create_node_ref::<html::Input>();
    let price_ref = create_node_ref::<html::Input>();
    let stock_ref = create_node_ref::<html::Input>();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = name_ref.get().unwrap().value();
        let price: f64 = price_ref.get().unwrap().value().parse().unwrap_or(0.0);
        let stock: u32 = stock_ref.get().unwrap().value().parse().unwrap_or(0);

        if name.is_empty() { return; }
        let payload = CreateItemPayload { name, price, stock };

        wasm_bindgen_futures::spawn_local(async move {
            let _ = Request::post(API_BASE).json(&payload).unwrap().send().await;
        });

        name_ref.get().unwrap().set_value("");
        price_ref.get().unwrap().set_value("");
        stock_ref.get().unwrap().set_value("");
    };

    let update_stock = move |id: u32, current_stock: u32, delta: i32| {
        let new_stock = if delta < 0 {
            if current_stock == 0 { return; }
            current_stock - 1
        } else {
            current_stock + 1
        };

        let payload = UpdateItemPayload {
            price: None,
            stock: Some(new_stock),
        };

        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}/{}", API_BASE, id);
            let _ = Request::put(&url).json(&payload).unwrap().send().await;
        });
    };

    let delete_item = move |id: u32| {
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}/{}", API_BASE, id);
            let _ = Request::delete(&url).send().await;
        });
    };

    let download_csv = move |items: Vec<Item>| {
        let mut csv_text = String::from("ID,Nama Barang,Harga,Stok\n");
        for item in items {
            let escaped_name = item.name.replace('"', "\"\"");
            csv_text.push_str(&format!("{},\"{}\",{},{}\n", item.id, escaped_name, item.price, item.stock));
        }

        let array = js_sys::Array::new();
        array.push(&wasm_bindgen::JsValue::from_str(&csv_text));

        let mut blob_props = web_sys::BlobPropertyBag::new();
        blob_props.type_("text/csv;charset=utf-8;");

        if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &blob_props) {
            if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Ok(element) = doc.create_element("a") {
                        let anchor: HtmlAnchorElement = element.unchecked_into();
                        anchor.set_href(&url);
                        anchor.set_download("inventaris.csv");
                        anchor.click();
                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                }
            }
        }
    };

    view! {
        <main class="max-w-4xl mx-auto p-6 space-y-8">
            // Header & Tombol Snapshot Backup
            <header class="border-b pb-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                <div>
                    <h1 class="text-3xl font-extrabold text-indigo-700">"Rust Fullstack WASM"</h1>
                    <p class="text-gray-500 mt-1">"Dashboard Inventaris Client-Side Rendered dengan Leptos & Axum"</p>
                </div>
                
                <a 
                    href="/api/backup" 
                    download="inventory-backup.redb"
                    class="inline-flex items-center gap-2 bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold py-2 px-3.5 rounded shadow-sm transition"
                >
                    "💾 Backup Database"
                </a>
            </header>

            // Form Tambah Barang Manual
            <section class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
                <h2 class="text-lg font-semibold text-gray-700 mb-4">"Tambah Barang Manual"</h2>
                <form on:submit=on_submit class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <input 
                        type="text" 
                        node_ref=name_ref 
                        placeholder="Nama Barang" 
                        class="border rounded px-3 py-2 text-sm focus:outline-indigo-500" 
                        required 
                    />
                    <input 
                        type="number" 
                        node_ref=price_ref 
                        placeholder="Harga (Rp)" 
                        class="border rounded px-3 py-2 text-sm focus:outline-indigo-500" 
                        required 
                    />
                    <input 
                        type="number" 
                        node_ref=stock_ref 
                        placeholder="Jumlah Stok" 
                        class="border rounded px-3 py-2 text-sm focus:outline-indigo-500" 
                        required 
                    />
                    <button 
                        type="submit" 
                        class="bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded text-sm transition"
                    >
                        "+ Tambah Data"
                    </button>
                </form>
            </section>

            // Tabel Daftar Inventaris Utama
            <section class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200 flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <h2 class="text-lg font-semibold text-gray-700">"Daftar Inventaris"</h2>
                    
                    <div class="flex items-center gap-2">
                        <input 
                            type="text" 
                            placeholder="🔍 Cari barang..." 
                            class="border rounded px-3 py-1.5 text-sm focus:outline-indigo-500 w-40 md:w-56" 
                            on:input=move |ev| set_search_query.set(event_target_value(&ev)) 
                            prop:value=search_query 
                        />
                        
                        {move || {
                            items_resource.get().map(|opt| {
                                if let Some(items) = opt {
                                    let items_clone = items.clone();
                                    view! {
                                        <button 
                                            on:click=move |_| download_csv(items_clone.clone())
                                            class="text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 py-2 px-3 rounded transition font-medium"
                                        >
                                            "📊 Export CSV"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }
                            })
                        }}

                        <button 
                            on:click=move |_| set_trigger.update(|n| *n += 1) 
                            class="text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 py-2 px-3 rounded transition"
                        >
                            "🔄 Refresh"
                        </button>
                    </div>
                </div>

                <Suspense fallback=move || view! { <p class="p-6 text-center text-gray-500">"Memuat data dari Axum..."</p> }>
                    {move || {
                        items_resource.get().map(|items_opt| {
                            match items_opt {
                                Some(items) if items.is_empty() => {
                                    view! { <p class="p-6 text-center text-gray-400">"Belum ada barang di database."</p> }.into_view()
                                },
                                Some(items) => {
                                    let query = search_query.get().to_lowercase();
                                    let filtered_items = items.into_iter()
                                        .filter(|item| query.is_empty() || item.name.to_lowercase().contains(&query))
                                        .collect::<Vec<_>>();

                                    if filtered_items.is_empty() {
                                        return view! { 
                                            <p class="p-6 text-center text-gray-400">
                                                {format!("Tidak ada barang yang cocok dengan kata kunci \"{}\"", query)}
                                            </p> 
                                        }.into_view();
                                    }

                                    view! {
                                        <table class="w-full text-left border-collapse">
                                            <thead>
                                                <tr class="bg-gray-50 text-gray-600 text-xs uppercase font-semibold">
                                                    <th class="py-3 px-6 text-center">"ID"</th>
                                                    <th class="py-3 px-6">"Nama Barang"</th>
                                                    <th class="py-3 px-6 text-right">"Harga"</th>
                                                    <th class="py-3 px-6 text-center">"Stok"</th>
                                                    <th class="py-3 px-6 text-center">"Aksi"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-200 text-sm">
                                                {filtered_items.into_iter().map(|item| {
                                                    let id = item.id;
                                                    let current_stock = item.stock;
                                                    let stock_badge_class = if current_stock <= 5 {
                                                        "bg-red-100 text-red-700 font-bold"
                                                    } else {
                                                        "bg-blue-100 text-blue-800"
                                                    };

                                                    view! {
                                                        <tr class="hover:bg-gray-50 transition">
                                                            <td class="py-3 px-6 text-center font-mono text-gray-500">{item.id}</td>
                                                            <td class="py-3 px-6 font-medium text-gray-900">{item.name}</td>
                                                            <td class="py-3 px-6 text-right text-emerald-600 font-semibold">{format!("Rp{:.0}", item.price)}</td>
                                                            <td class="py-3 px-6 text-center">
                                                                <div class="inline-flex items-center gap-2">
                                                                    <button 
                                                                        on:click=move |_| update_stock(id, current_stock, -1)
                                                                        class="w-6 h-6 rounded bg-gray-200 hover:bg-gray-300 text-gray-700 font-bold flex items-center justify-center text-xs transition"
                                                                    >
                                                                        "-"
                                                                    </button>
                                                                    <span class={format!("px-2.5 py-0.5 text-xs rounded-full min-w-[2rem] inline-block {}", stock_badge_class)}>
                                                                        {item.stock}
                                                                    </span>
                                                                    <button 
                                                                        on:click=move |_| update_stock(id, current_stock, 1)
                                                                        class="w-6 h-6 rounded bg-gray-200 hover:bg-gray-300 text-gray-700 font-bold flex items-center justify-center text-xs transition"
                                                                    >
                                                                        "+"
                                                                    </button>
                                                                </div>
                                                            </td>
                                                            <td class="py-3 px-6 text-center">
                                                                <button 
                                                                    on:click=move |_| delete_item(id)
                                                                    class="text-xs bg-red-50 hover:bg-red-100 text-red-600 font-medium py-1 px-3 rounded transition"
                                                                >
                                                                    "Hapus"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    }.into_view()
                                },
                                None => view! { <p class="p-6 text-center text-red-500">"Gagal tersambung ke backend API."</p> }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </section>

            // Tabel Riwayat Transaksi (Audit Logs)
            <section class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold text-gray-700">"Riwayat Transaksi (Audit Logs)"</h2>
                    <p class="text-xs text-gray-400 mt-0.5">"Log mutasi data otomatis tersimpan di tabel audit_logs database Redb"</p>
                </div>

                <Suspense fallback=move || view! { <p class="p-6 text-center text-gray-500">"Memuat audit logs..."</p> }>
                    {move || {
                        logs_resource.get().map(|logs_opt| {
                            match logs_opt {
                                Some(logs) if logs.is_empty() => {
                                    view! { <p class="p-6 text-center text-gray-400">"Belum ada catatan aktivitas transaksi."</p> }.into_view()
                                },
                                Some(logs) => {
                                    view! {
                                        <div class="max-h-72 overflow-y-auto">
                                            <table class="w-full text-left border-collapse text-sm">
                                                <thead class="sticky top-0 bg-gray-50 text-gray-600 text-xs uppercase font-semibold">
                                                    <tr>
                                                        <th class="py-3 px-6 text-center">"Item ID"</th>
                                                        <th class="py-3 px-6">"Tipe Aksi"</th>
                                                        <th class="py-3 px-6 text-center">"Delta Stok"</th>
                                                        <th class="py-3 px-6 text-right">"Timestamp (Epoch)"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-200 text-xs font-mono">
                                                    {logs.into_iter().map(|log| {
                                                        let (type_badge, delta_badge) = match log.change_type.as_str() {
                                                            "Created" => ("bg-blue-100 text-blue-700", "text-blue-600 font-bold"),
                                                            "Deleted" => ("bg-red-100 text-red-700", "text-red-600 font-bold"),
                                                            _ => ("bg-amber-100 text-amber-700", if log.delta > 0 { "text-emerald-600 font-bold" } else { "text-rose-600 font-bold" }),
                                                        };

                                                        let delta_text = if log.delta > 0 {
                                                            format!("+{}", log.delta)
                                                        } else {
                                                            format!("{}", log.delta)
                                                        };

                                                        view! {
                                                            <tr class="hover:bg-gray-50 transition">
                                                                <td class="py-2.5 px-6 text-center font-bold text-gray-600">{log.item_id}</td>
                                                                <td class="py-2.5 px-6 font-sans">
                                                                    <span class={format!("px-2 py-0.5 rounded text-[11px] font-semibold {}", type_badge)}>
                                                                        {log.change_type}
                                                                    </span>
                                                                </td>
                                                                <td class={format!("py-2.5 px-6 text-center {}", delta_badge)}>
                                                                    {delta_text}
                                                                </td>
                                                                <td class="py-2.5 px-6 text-right text-gray-400">{log.timestamp}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_view()
                                },
                                None => view! { <p class="p-6 text-center text-gray-400">"Data log audit belum tersedia."</p> }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </section>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
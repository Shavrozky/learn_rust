use gloo_net::http::Request;
use leptos::*;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "http://127.0.0.1:3000/items";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub stock: u32,
}

#[derive(Serialize)]
struct CreateItemPayload {
    name: String,
    price: f64,
    stock: u32,
}

#[component]
fn App() -> impl IntoView {
    let (trigger, set_trigger) = create_signal(0);

    // Fetch data items dari backend Axum
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

    // Node references untuk form input
    let name_ref = create_node_ref::<html::Input>();
    let price_ref = create_node_ref::<html::Input>();
    let stock_ref = create_node_ref::<html::Input>();

    // Handler Tambah Barang
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = name_ref.get().unwrap().value();
        let price: f64 = price_ref.get().unwrap().value().parse().unwrap_or(0.0);
        let stock: u32 = stock_ref.get().unwrap().value().parse().unwrap_or(0);

        if name.is_empty() { return; }

        let payload = CreateItemPayload { name, price, stock };

        wasm_bindgen_futures::spawn_local(async move {
            let _ = Request::post(API_BASE)
                .json(&payload)
                .unwrap()
                .send()
                .await;
            
            // Refresh data tabel
            set_trigger.update(|n| *n += 1);
        });

        // Reset form input
        name_ref.get().unwrap().set_value("");
        price_ref.get().unwrap().set_value("");
        stock_ref.get().unwrap().set_value("");
    };

    // Handler Hapus Barang
    let delete_item = move |id: u32| {
        wasm_bindgen_futures::spawn_local(async move {
            let url = format!("{}/{}", API_BASE, id);
            let _ = Request::delete(&url).send().await;
            set_trigger.update(|n| *n += 1);
        });
    };

    view! {
        <main class="max-w-4xl mx-auto p-6">
            <header class="mb-8 border-b pb-4">
                <h1 class="text-3xl font-extrabold text-indigo-700">"Rust Fullstack WASM"</h1>
                <p class="text-gray-500 mt-1">"Dashboard Inventaris Client-Side Rendered dengan Leptos & Axum"</p>
            </header>

            // Form Tambah Barang
            <section class="bg-white p-6 rounded-lg shadow-sm border border-gray-200 mb-8">
                <h2 class="text-lg font-semibold text-gray-700 mb-4">"Tambah Barang Baru"</h2>
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

            // Tabel Daftar Barang
            <section class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
                    <h2 class="text-lg font-semibold text-gray-700">"Daftar Inventaris"</h2>
                    <button 
                        on:click=move |_| set_trigger.update(|n| *n += 1)
                        class="text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 py-1 px-3 rounded"
                    >
                        "🔄 Refresh"
                    </button>
                </div>

                <Suspense fallback=move || view! { <p class="p-6 text-center text-gray-500">"Memuat data dari Axum..."</p> }>
                    {move || {
                        items_resource.get().map(|items_opt| {
                            match items_opt {
                                Some(items) if items.is_empty() => {
                                    view! { <p class="p-6 text-center text-gray-400">"Belum ada barang di database."</p> }.into_view()
                                },
                                Some(items) => {
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
                                                {items.into_iter().map(|item| {
                                                    let id = item.id;
                                                    view! {
                                                        <tr class="hover:bg-gray-50 transition">
                                                            <td class="py-3 px-6 text-center font-mono text-gray-500">{item.id}</td>
                                                            <td class="py-3 px-6 font-medium text-gray-900">{item.name}</td>
                                                            <td class="py-3 px-6 text-right text-emerald-600 font-semibold">{format!("Rp{:.0}", item.price)}</td>
                                                            <td class="py-3 px-6 text-center">
                                                                <span class="px-2 py-1 text-xs font-semibold rounded-full bg-blue-100 text-blue-800">
                                                                    {item.stock}
                                                                </span>
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
                                None => view! { <p class="p-6 text-center text-red-500">"Gagal tersambung ke backend API (Port 3000)."</p> }.into_view()
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
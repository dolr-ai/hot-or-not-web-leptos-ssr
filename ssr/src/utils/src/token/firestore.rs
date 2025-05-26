use std::sync::{Arc, Mutex};

use js_sys::Object;
use serde::{Deserialize, Serialize};

use futures::stream::BoxStream;

use futures::channel::mpsc;
use wasm_bindgen::prelude::*;
use web_time::{Duration, SystemTime};

use super::icpump::TokenListItem;

#[wasm_bindgen(module = "/src/token/icpump-inline.js")]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    pub type FirebaseApp;
    pub type Firestore;
    pub type QuerySnapshot;
    pub type QueryDocumentSnapshot;

    #[wasm_bindgen(catch, js_namespace = ["firebase"])]
    fn initializeApp(config: &JsValue) -> Result<FirebaseApp, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["firebase"])]
    fn getFirestore(app: &FirebaseApp, cf: &str) -> Result<Firestore, JsValue>;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"])]
    fn collection(firestore: &Firestore, name: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"])]
    fn query(collection_ref: &JsValue, where_clause: &JsValue, order_clause: &JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"])]
    fn limit(limit: u32) -> JsValue;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"])]
    fn getDocs(query: &JsValue) -> js_sys::Promise;

    #[wasm_bindgen(method, getter)]
    fn docs(this: &QuerySnapshot) -> js_sys::Array;

    #[wasm_bindgen(method)]
    fn data(this: &QueryDocumentSnapshot) -> JsValue;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"])]
    fn onSnapshot(query: &JsValue, callback: &Closure<dyn Fn(QuerySnapshot)>) -> js_sys::Function;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"], js_name = where)]
    fn r#where(field: &str, op: &str, value: &JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore"])]
    fn orderBy(field: &str, ord: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["firebase", "firestore", "Timestamp"])]
    fn now() -> JsValue;
}

fn get_firebase_config() -> JsValue {
    let config = Object::new();
    js_sys::Reflect::set(
        &config,
        &"apiKey".into(),
        &"AIzaSyCwo0EWTJz_w-J1lUf9w9NcEBdLNmGUaIo".into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &config,
        &"authDomain".into(),
        &"hot-or-not-feed-intelligence.firebaseapp.com".into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &config,
        &"databaseURL".into(),
        &"https://hot-or-not-feed-intelligence-default-rtdb.firebaseio.com".into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &config,
        &"projectId".into(),
        &"hot-or-not-feed-intelligence".into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &config,
        &"storageBucket".into(),
        &"hot-or-not-feed-intelligence.appspot.com".into(),
    )
    .unwrap();
    js_sys::Reflect::set(&config, &"messagingSenderId".into(), &"82502260393".into()).unwrap();
    js_sys::Reflect::set(
        &config,
        &"appId".into(),
        &"1:82502260393:web:21f1fa48319fbc5e237bb8".into(),
    )
    .unwrap();
    config.into()
}

// Initialize Firebase app and Firestore
pub fn init_firebase() -> (FirebaseApp, Firestore) {
    let config = get_firebase_config();

    let app = initializeApp(&config).unwrap();
    let firestore = getFirestore(&app, "ic-pump-fun").unwrap();
    (app, firestore)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeEpoch {
    pub seconds: u64,
    pub nanoseconds: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenFirestoreBindingsItem {
    pub user_id: String,
    pub name: String,
    pub token_name: String,
    pub token_symbol: String,
    pub logo: String,
    pub description: String,
    pub created_at: TimeEpoch,
    #[serde(default)]
    pub link: String,
}

impl From<TokenFirestoreBindingsItem> for TokenListItem {
    fn from(item: TokenFirestoreBindingsItem) -> Self {
        let timestamp = SystemTime::UNIX_EPOCH
            + Duration::new(item.created_at.seconds, item.created_at.nanoseconds);
        let now = SystemTime::now();
        let elapsed = now.duration_since(timestamp).unwrap().as_secs();

        let elapsed_str = if elapsed < 60 {
            format!("{elapsed}s ago")
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else if elapsed < 86400 {
            format!("{}h ago", elapsed / 3600)
        } else {
            format!("{}d ago", elapsed / 86400)
        };

        TokenListItem {
            user_id: item.user_id,
            name: item.name,
            token_name: item.token_name,
            token_symbol: item.token_symbol,
            logo: item.logo,
            description: item.description,
            created_at: "".to_string(),
            formatted_created_at: elapsed_str,
            link: item.link,
            is_nsfw: false,
        }
    }
}

pub fn listen_to_documents(firestore: &Firestore) -> BoxStream<'static, Vec<TokenListItem>> {
    let (sender, receiver) = mpsc::channel(100);

    let collection_ref = collection(firestore, "tokens-list");

    let now_timestamp = now();
    let where_clause = r#where("created_at", ">", &now_timestamp);
    let order_clause = orderBy("created_at", "asc");
    let query_ref = query(&collection_ref, &where_clause, &order_clause);

    let last_doc_info = Arc::new(Mutex::new(0 as f64));

    let callback = Closure::wrap(Box::new(move |snapshot: QuerySnapshot| {
        let mut last_doc_created_by = last_doc_info.lock().unwrap();

        let new_docs: Vec<TokenListItem> = snapshot
            .docs()
            .iter()
            .filter_map(|doc| {
                let doc: QueryDocumentSnapshot = doc.dyn_into().unwrap();
                let data: TokenFirestoreBindingsItem =
                    serde_wasm_bindgen::from_value(doc.data()).unwrap();

                let created_at = data.created_at.seconds as f64 * 1000.0
                    + (data.created_at.nanoseconds as f64 / 1_000_000.0);

                if *last_doc_created_by != 0.0 && created_at <= *last_doc_created_by {
                    return None;
                }

                *last_doc_created_by = created_at;

                let data: TokenListItem = data.into();

                Some(data)
            })
            .collect();

        let mut sender = sender.clone();
        sender.try_send(new_docs).unwrap();
    }) as Box<dyn Fn(QuerySnapshot)>);

    onSnapshot(&query_ref, &callback);
    callback.forget(); // Prevent the closure from being dropped

    Box::pin(receiver)
}

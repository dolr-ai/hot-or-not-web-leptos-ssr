use gloo_utils::format::JsValueSerdeExt;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use js_sys::Reflect;
use leptos::{prelude::*, task::spawn_local};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

#[server]
async fn check_if_enquiry_completed(
    inquiry_id: String,
    // reference_id: String,
) -> Result<bool, ServerFnError> {
    let token = std::env::var("KYC_SERVER_TOKEN").expect("KYC_SERVER_TOKEN is not set");
    let url = format!("https://api.withpersona.com/api/v1/inquiries/{inquiry_id}",);

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !res.status().is_success() {
        return Err(ServerFnError::ServerError(format!(
            "Failed to fetch inquiry: {}",
            res.status()
        )));
    }

    let data: InquiryData = res
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let status = data.data.attributes.status.to_lowercase();

    // let res_reference_id = data.data.attributes.reference_id;

    Ok(status == "approved")
}

#[derive(Debug, Deserialize)]
struct InquiryData {
    data: Inquiry,
}

#[derive(Debug, Deserialize)]
struct Inquiry {
    attributes: InquiryAttributes,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct InquiryAttributes {
    status: String,
    #[serde(rename = "reference-id")]
    reference_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KycResult {
    #[serde(rename = "inquiryId")]
    pub inquiry_id: String,
    pub status: String,
    pub fields: Value,
}

pub fn kyc_on_status_change(status: String) {
    let parsed_status = match status.as_str() {
        "InProgress" => KycStatus::InProgress,
        _ => KycStatus::Pending,
    };

    KycState::set_status(parsed_status);
}

pub fn kyc_on_complete(kyc_result: JsValue) {
    match kyc_result.into_serde::<KycResult>() {
        Ok(result) => {
            let parsed_status = match result.status.as_str() {
                "approved" => KycStatus::Verified,
                "in_progress" => KycStatus::InProgress,
                _ => KycStatus::Pending,
            };
            if parsed_status == KycStatus::Verified {
                // Check if the inquiry is completed
                let inquiry_id = result.inquiry_id.clone();
                spawn_local(async move {
                    if check_if_enquiry_completed(inquiry_id)
                        .await
                        .unwrap_or(false)
                    {
                        KycState::set_status(KycStatus::Verified);
                    } else {
                        KycState::set_status(KycStatus::Pending);
                    }
                });
            } else {
                KycState::set_status(parsed_status);
            }
        }
        Err(_) => {
            KycState::set_status(KycStatus::Pending);
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaConfig<'a> {
    template_id: &'a str,
    reference_id: &'a str,
    environment_id: &'a str,
}

impl<'a> PersonaConfig<'a> {
    pub fn launch(user_principal: &'a str) {
        let config = Self {
            template_id: "itmpl_dJb7DuMvNSgg8dQEwYF3zh5dD8Nm",
            reference_id: user_principal,
            environment_id: "sandbox",
        };

        let js_config = JsValue::from_serde(&config).expect("Failed to serialize PersonaConfig");

        let callback = Closure::wrap(Box::new(kyc_on_status_change) as Box<dyn Fn(String)>);
        let callback_completed = Closure::wrap(Box::new(kyc_on_complete) as Box<dyn Fn(JsValue)>);

        let window = window();
        let func = Reflect::get(&window, &JsValue::from_str("launchPersonaFlow"))
            .expect("launchPersonaFlow not defined");

        let func: &js_sys::Function = func.dyn_ref().expect("launchPersonaFlow is not a function");

        let _ = func
            .call3(
                &JsValue::NULL,
                &js_config,
                callback.as_ref().unchecked_ref(),
                callback_completed.as_ref().unchecked_ref(),
            )
            .expect("Failed to call launchPersonaFlow");

        callback.forget();
        callback_completed.forget();
    }
}

#[derive(Clone, Default, PartialEq)]
pub enum KycStatus {
    #[default]
    Pending,
    InProgress,
    Verified,
}

#[derive(Default, Clone)]
pub struct KycState {
    pub kyc_status: RwSignal<KycStatus>,
}

impl KycState {
    pub fn init() -> Self {
        let this = Self {
            ..Default::default()
        };
        provide_context(this.clone());
        this
    }

    fn with_ctx() -> Self {
        use_context::<Self>().unwrap_or_else(KycState::init)
    }

    pub fn get() -> RwSignal<KycStatus> {
        Self::with_ctx().kyc_status
    }

    pub fn is_verified() -> bool {
        let this = Self::with_ctx();
        this.kyc_status.get() == KycStatus::Verified
    }

    pub fn set_status(status: KycStatus) {
        let this = Self::with_ctx();
        this.kyc_status.set(status);
    }

    pub fn toggle() {
        let this = Self::with_ctx();
        this.kyc_status.update(|v| {
            *v = match *v {
                KycStatus::Pending => KycStatus::InProgress,
                KycStatus::InProgress => KycStatus::Verified,
                KycStatus::Verified => KycStatus::Pending,
            };
        });
    }
}

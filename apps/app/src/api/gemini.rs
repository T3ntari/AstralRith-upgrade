use crate::api::Result;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_http::reqwest;
use tokio::sync::oneshot;

fn gemini_error(message: impl Into<String>) -> theseus::Error {
    theseus::ErrorKind::OtherError(message.into()).into()
}

pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("gemini")
        .invoke_handler(tauri::generate_handler![
            gemini_list_models,
            gemini_generate_start,
            gemini_generate_cancel
        ])
        .build()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeminiModel {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: u64,
    pub output_token_limit: u64,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub version: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiModelsResponse {
    models: Option<Vec<GeminiModelRaw>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiModelRaw {
    name: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    description: Option<String>,
    #[serde(rename = "inputTokenLimit")]
    input_token_limit: u64,
    #[serde(rename = "outputTokenLimit")]
    output_token_limit: u64,
    supported_generation_methods: Option<Vec<String>>,
    temperature: Option<f64>,
    #[serde(rename = "topP")]
    top_p: Option<f64>,
    #[serde(rename = "topK")]
    top_k: Option<u32>,
    version: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerateResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponseContent {
    parts: Option<Vec<GeminiPart>>,
}

lazy_static::lazy_static! {
    static ref ACTIVE_REQUESTS: Mutex<HashMap<String, oneshot::Sender<()>>> =
        Mutex::new(HashMap::new());
}

/// Lists all Gemini models that support text/multimodal generation
// invoke('plugin:gemini|gemini_list_models', { apiKey })
#[tauri::command]
pub async fn gemini_list_models(api_key: String) -> Result<Vec<GeminiModel>> {
    tracing::info!("Fetching Gemini models list");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );

    let response = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(gemini_error(format!("Gemini API error: {}", status)).into());
    }

    let parsed: GeminiModelsResponse = response.json().await?;

    let models = parsed
        .models
        .unwrap_or_default()
        .into_iter()
        .filter(|m| {
            m.supported_generation_methods
                .as_ref()
                .map(|methods| methods.iter().any(|m| m == "generateContent"))
                .unwrap_or(false)
        })
        .map(|m| {
            let name = m.name.clone();
            GeminiModel {
                name: m.name,
                display_name: m.display_name.unwrap_or_else(|| {
                    name.trim_start_matches("models/").to_string()
                }),
                description: m.description.unwrap_or_default(),
                input_token_limit: m.input_token_limit,
                output_token_limit: m.output_token_limit,
                temperature: m.temperature,
                top_p: m.top_p,
                top_k: m.top_k,
                version: m.version,
            }
        })
        .collect::<Vec<_>>();

    tracing::info!("Gemini models loaded: {}", models.len());
    Ok(models)
}

/// Starts a streaming generation. Emits `gemini-delta` events with
/// { requestId, text } for each chunk and { requestId, done: true } at the end.
// invoke('plugin:gemini|gemini_generate_start', { apiKey, modelName, contents, requestId })
#[tauri::command]
pub async fn gemini_generate_start(
    app: AppHandle,
    api_key: String,
    model_name: String,
    contents: Vec<GeminiContent>,
    request_id: String,
) -> Result<()> {
    tracing::info!("Starting Gemini generation with model {model_name}");
    let (abort_tx, abort_rx) = oneshot::channel::<()>();
    {
        let mut requests = ACTIVE_REQUESTS.lock().unwrap();
        requests.insert(request_id.clone(), abort_tx);
    }

    let app_clone = app.clone();
    let model = model_name.trim_start_matches("models/");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    tauri::async_runtime::spawn(async move {
        let result = stream_generation(app_clone.clone(), &url, contents, abort_rx, &request_id).await;
        if let Err(err) = result {
            let _ = app_clone.emit(
                "gemini-delta",
                serde_json::json!({
                    "requestId": request_id,
                    "error": err.to_string(),
                }),
            );
        }
        ACTIVE_REQUESTS.lock().unwrap().remove(&request_id);
    });

    Ok(())
}

/// Cancels a running generation
// invoke('plugin:gemini|gemini_generate_cancel', { requestId })
#[tauri::command]
pub fn gemini_generate_cancel(request_id: String) -> Result<()> {
    if let Some(tx) = ACTIVE_REQUESTS.lock().unwrap().remove(&request_id) {
        let _ = tx.send(());
    }
    Ok(())
}

async fn stream_generation(
    app: AppHandle,
    url: &str,
    contents: Vec<GeminiContent>,
    mut abort_rx: oneshot::Receiver<()>,
    request_id: &str,
) -> crate::api::Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Accept", "text/event-stream")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "contents": contents }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(gemini_error(format!(
            "Gemini API error: {}",
            response.status()
        ))
        .into());
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    let result: crate::api::Result<()> = async {
        loop {
            let chunk = tokio::select! {
                res = stream.next() => match res {
                    Some(Ok(chunk)) => chunk,
                    Some(Err(err)) => {
                        return Err(gemini_error(format!("Failed reading Gemini stream: {err}")).into())
                    }
                    None => break,
                },
                _ = &mut abort_rx => break,
            };

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(newline) = buffer.find('\n') {
                let line = buffer[..newline].trim().to_string();
                buffer.drain(..=newline);

                if !line.starts_with("data:") {
                    continue;
                }
                let payload = line[5..].trim().to_string();
                if payload.is_empty() {
                    continue;
                }

                let Ok(parsed) = serde_json::from_str::<GeminiGenerateResponse>(&payload) else {
                    continue;
                };

                let parts = parsed
                    .candidates
                    .and_then(|c| c.into_iter().next())
                    .and_then(|c| c.content)
                    .and_then(|c| c.parts)
                    .unwrap_or_default();

                let text = parts
                    .into_iter()
                    .map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("");

                if !text.is_empty() {
                    app.emit(
                        "gemini-delta",
                        serde_json::json!({
                            "requestId": request_id,
                            "text": text,
                        }),
                    )?;
                }
            }
        }

        app.emit(
            "gemini-delta",
            serde_json::json!({
                "requestId": request_id,
                "done": true,
            }),
        )?;

        Ok(())
    }
    .await;

    result
}

use crate::summarizer;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Deserialize)]
struct PartResponse {
    text: String,
}

fn load_summarizer(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    summarizer::build_input(path).map_err(|e| {
        eprintln!("Error building input: {}", e);
        e.into()
    })
}

pub async fn generate_md(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let api_key =
        env::var("GEMINI_API_KEY").expect("Please set the GEMINI_API_KEY environment variable.");

    let prompt = match load_summarizer(path.clone()) {
        Ok(p) => p,
        Err(_) => {
            return Err("Failed to generate prompt from summarizer".into());
        }
    };

    let request_body = GeminiRequest {
        contents: vec![Content {
            role: "user".to_string(),
            parts: vec![Part { text: prompt }],
        }],
    };

    let client = Client::new();

    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
            api_key
        ))
        .json(&request_body)
        .send()
        .await?;

    let response_data: GeminiResponse = response.json().await?;

    if let Some(part) = response_data
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
    {
        Ok(part.text.clone())
    } else {
        Err("No markdown content generated".into())
    }
}

use crate::summarizer;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Serialize)]
struct GeminiRequest {
    prompt: String,
    max_tokens: u16,
}

#[derive(Deserialize)]
struct GeminiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

fn load_summarizer(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    summarizer::build_input(path).map_err(|e| {
        eprintln!("Error building input: {}", e);
        e.into()
    })
}

async fn generate_md(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        env::var("GEMINI_API_KEY").expect("Please set the GEMINI_API_KEY environment variable.");

    let mut request_body = GeminiRequest {
        prompt: String::new(),
        max_tokens: 100,
    };
    if let Some(prompt) = load_summarizer(path.clone()).ok() {
        request_body.prompt = prompt;
    }

    let client = Client::new();
    let response = client
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key))
        .json(&request_body)
        .send()
        .await?;

    let response_data: GeminiResponse = response.json().await?;
    if let Some(choice) = response_data.choices.first() {
        println!("Repo Markdown:\n{}", choice.text);
    }
    Ok(())
}

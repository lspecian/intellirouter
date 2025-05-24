// Simple Client Example for IntelliRouter
//
// This example demonstrates how to create a Rust client that connects to IntelliRouter
// and sends a chat completion request.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

// Define the request and response structures based on the OpenAI API format
// that IntelliRouter implements

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionChoice {
    index: u32,
    message: ChatCompletionMessage,
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<ChatCompletionChoice>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("IntelliRouter Simple Client Example");
    println!("==================================");

    // Configuration
    let host = "localhost";
    let port = 8080;
    let model = "gpt-3.5-turbo";
    let endpoint = format!("http://{}:{}/v1/chat/completions", host, port);

    println!("Connecting to IntelliRouter at {}", endpoint);

    // Create a client
    let client = Client::new();

    // Create a request
    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "What is the capital of France?".to_string(),
        }],
        stream: false,
    };

    println!("Sending request to IntelliRouter...");
    println!("Question: What is the capital of France?");

    // Send the request
    let response = client.post(&endpoint).json(&request).send().await?;

    // Check if the request was successful
    if response.status().is_success() {
        // Parse the response
        let completion: ChatCompletionResponse = response.json().await?;

        // Extract and display the response
        if let Some(choice) = completion.choices.first() {
            println!("\nResponse from model {}:", completion.model);
            println!("{}", choice.message.content);
        } else {
            println!("No response content received");
        }
    } else {
        println!("Error: {}", response.status());
        println!("Response: {}", response.text().await?);
    }

    println!("\nExample completed");
    Ok(())
}

// Note: To run this example, you need to have IntelliRouter running.
// Start IntelliRouter with: intellirouter run --role router
//
// Then run this example with: cargo run --example simple_client
//
// Dependencies required in Cargo.toml:
// reqwest = { version = "0.11", features = ["json"] }
// serde = { version = "1.0", features = ["derive"] }
// tokio = { version = "1.0", features = ["full"] }

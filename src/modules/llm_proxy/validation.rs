//! LLM Proxy Request Validation
//!
//! This module implements validation logic for incoming API requests to ensure
//! they meet the required format and constraints before processing.

use super::routes::{ApiError, ChatCompletionRequest, ChatMessage};

/// Validate a chat completion request
pub fn validate_chat_completion_request(request: &ChatCompletionRequest) -> Result<(), ApiError> {
    // Validate model
    validate_model(&request.model)?;

    // Validate messages
    validate_messages(&request.messages)?;

    // Validate temperature
    if let Some(temp) = request.temperature {
        if temp < 0.0 || temp > 2.0 {
            return Err(create_validation_error(
                "temperature must be between 0.0 and 2.0",
                Some("temperature"),
            ));
        }
    }

    // Validate top_p
    if let Some(top_p) = request.top_p {
        if top_p < 0.0 || top_p > 1.0 {
            return Err(create_validation_error(
                "top_p must be between 0.0 and 1.0",
                Some("top_p"),
            ));
        }
    }

    // Validate n (number of completions)
    if let Some(n) = request.n {
        if n == 0 || n > 10 {
            return Err(create_validation_error(
                "n must be between 1 and 10",
                Some("n"),
            ));
        }
    }

    // Validate max_tokens
    if let Some(max_tokens) = request.max_tokens {
        if max_tokens == 0 {
            return Err(create_validation_error(
                "max_tokens must be greater than 0",
                Some("max_tokens"),
            ));
        }

        // A reasonable upper limit for most models
        if max_tokens > 8192 {
            return Err(create_validation_error(
                "max_tokens exceeds the maximum allowed value of 8192",
                Some("max_tokens"),
            ));
        }
    }

    // Validate presence_penalty
    if let Some(penalty) = request.presence_penalty {
        if penalty < -2.0 || penalty > 2.0 {
            return Err(create_validation_error(
                "presence_penalty must be between -2.0 and 2.0",
                Some("presence_penalty"),
            ));
        }
    }

    // Validate frequency_penalty
    if let Some(penalty) = request.frequency_penalty {
        if penalty < -2.0 || penalty > 2.0 {
            return Err(create_validation_error(
                "frequency_penalty must be between -2.0 and 2.0",
                Some("frequency_penalty"),
            ));
        }
    }

    Ok(())
}

/// Validate the model name
fn validate_model(model: &str) -> Result<(), ApiError> {
    // Check if model is empty
    if model.trim().is_empty() {
        return Err(create_validation_error(
            "model is required and cannot be empty",
            Some("model"),
        ));
    }

    // Check if model is supported
    // This will be expanded in future tasks when model registry is implemented
    // For now, just check if it's a known format
    let supported_prefixes = ["gpt-", "text-", "claude-", "mistral-", "llama-"];
    if !supported_prefixes
        .iter()
        .any(|prefix| model.starts_with(prefix))
    {
        return Err(create_validation_error(
            &format!("model '{}' is not supported", model),
            Some("model"),
        ));
    }

    Ok(())
}

/// Validate the messages array
fn validate_messages(messages: &[ChatMessage]) -> Result<(), ApiError> {
    // Check if messages is empty
    if messages.is_empty() {
        return Err(create_validation_error(
            "messages is required and cannot be empty",
            Some("messages"),
        ));
    }

    // Validate each message
    for (i, message) in messages.iter().enumerate() {
        // Validate role
        match message.role.as_str() {
            "system" | "user" | "assistant" | "function" => {}
            _ => {
                return Err(create_validation_error(
                    &format!("invalid role '{}' at messages[{}], must be one of: system, user, assistant, function", message.role, i),
                    Some("messages"),
                ));
            }
        }

        // Validate content
        if message.content.trim().is_empty() {
            return Err(create_validation_error(
                &format!("content cannot be empty at messages[{}]", i),
                Some("messages"),
            ));
        }

        // Validate name if present
        if let Some(name) = &message.name {
            if name.trim().is_empty() {
                return Err(create_validation_error(
                    &format!("name cannot be empty at messages[{}]", i),
                    Some("messages"),
                ));
            }

            // Check name length
            if name.len() > 64 {
                return Err(create_validation_error(
                    &format!(
                        "name at messages[{}] exceeds maximum length of 64 characters",
                        i
                    ),
                    Some("messages"),
                ));
            }
        }
    }

    // Validate message sequence
    // Check if there's at least one user message
    if !messages.iter().any(|m| m.role == "user") {
        return Err(create_validation_error(
            "messages must contain at least one user message",
            Some("messages"),
        ));
    }

    // Check if system message is first (if present)
    if messages
        .iter()
        .enumerate()
        .any(|(i, m)| m.role == "system" && i > 0)
    {
        return Err(create_validation_error(
            "system message must be the first message if present",
            Some("messages"),
        ));
    }

    Ok(())
}

/// Create a validation error
fn create_validation_error(message: &str, param: Option<&str>) -> ApiError {
    super::routes::handle_invalid_request(message, param)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_model() {
        // Valid models
        assert!(validate_model("gpt-3.5-turbo").is_ok());
        assert!(validate_model("text-davinci-003").is_ok());
        assert!(validate_model("claude-2").is_ok());
        assert!(validate_model("mistral-medium").is_ok());

        // Invalid models
        let err = validate_model("").unwrap_err();
        assert!(err.error.message.contains("cannot be empty"));

        let err = validate_model("invalid-model").unwrap_err();
        assert!(err.error.message.contains("not supported"));
    }

    #[test]
    fn test_validate_messages() {
        // Valid messages
        let valid_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            name: None,
        }];
        assert!(validate_messages(&valid_messages).is_ok());

        // Valid messages with system first
        let valid_system_first = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
                name: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            },
        ];
        assert!(validate_messages(&valid_system_first).is_ok());

        // Empty messages
        let empty_messages: Vec<ChatMessage> = vec![];
        let err = validate_messages(&empty_messages).unwrap_err();
        assert!(err.error.message.contains("cannot be empty"));

        // Invalid role
        let invalid_role = vec![ChatMessage {
            role: "invalid".to_string(),
            content: "Hello".to_string(),
            name: None,
        }];
        let err = validate_messages(&invalid_role).unwrap_err();
        assert!(err.error.message.contains("invalid role"));

        // Empty content
        let empty_content = vec![ChatMessage {
            role: "user".to_string(),
            content: "".to_string(),
            name: None,
        }];
        let err = validate_messages(&empty_content).unwrap_err();
        assert!(err.error.message.contains("content cannot be empty"));

        // No user message
        let no_user = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
                name: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "How can I help you?".to_string(),
                name: None,
            },
        ];
        let err = validate_messages(&no_user).unwrap_err();
        assert!(err.error.message.contains("at least one user message"));

        // System message not first
        let system_not_first = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            },
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
                name: None,
            },
        ];
        let err = validate_messages(&system_not_first).unwrap_err();
        assert!(err
            .error
            .message
            .contains("system message must be the first"));
    }

    #[test]
    fn test_validate_chat_completion_request() {
        // Valid request
        let valid_request = ChatCompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(0.7),
            top_p: Some(0.9),
            n: Some(1),
            stream: false,
            max_tokens: Some(100),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            user: None,
        };
        assert!(validate_chat_completion_request(&valid_request).is_ok());

        // Invalid temperature
        let mut invalid_request = valid_request.clone();
        invalid_request.temperature = Some(3.0);
        let err = validate_chat_completion_request(&invalid_request).unwrap_err();
        assert!(err.error.message.contains("temperature must be between"));

        // Invalid top_p
        let mut invalid_request = valid_request.clone();
        invalid_request.top_p = Some(1.5);
        let err = validate_chat_completion_request(&invalid_request).unwrap_err();
        assert!(err.error.message.contains("top_p must be between"));

        // Invalid n
        let mut invalid_request = valid_request.clone();
        invalid_request.n = Some(0);
        let err = validate_chat_completion_request(&invalid_request).unwrap_err();
        assert!(err.error.message.contains("n must be between"));

        // Invalid max_tokens
        let mut invalid_request = valid_request.clone();
        invalid_request.max_tokens = Some(0);
        let err = validate_chat_completion_request(&invalid_request).unwrap_err();
        assert!(err
            .error
            .message
            .contains("max_tokens must be greater than"));

        // Invalid presence_penalty
        let mut invalid_request = valid_request.clone();
        invalid_request.presence_penalty = Some(3.0);
        let err = validate_chat_completion_request(&invalid_request).unwrap_err();
        assert!(err
            .error
            .message
            .contains("presence_penalty must be between"));

        // Invalid frequency_penalty
        let mut invalid_request = valid_request.clone();
        invalid_request.frequency_penalty = Some(-3.0);
        let err = validate_chat_completion_request(&invalid_request).unwrap_err();
        assert!(err
            .error
            .message
            .contains("frequency_penalty must be between"));
    }
}

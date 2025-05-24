# IntelliRouter Examples

This directory contains examples demonstrating how to use IntelliRouter. These examples are designed to be simple, clear, and easy to understand.

## Prerequisites

Before running these examples, make sure you have:

1. Installed IntelliRouter (see the main [README.md](../README.md) for installation instructions)
2. Set up any required API keys in your environment variables (e.g., `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`)
3. Rust toolchain installed (for Rust examples)

## Examples Overview

### 1. Basic Usage Shell Script (`basic_usage.sh`)

This script demonstrates how to start IntelliRouter and send basic requests using curl.

**To run:**

```bash
# Make the script executable
chmod +x examples/basic_usage.sh

# Run the script
./examples/basic_usage.sh
```

**Expected output:**
- Step-by-step instructions for starting IntelliRouter
- Example curl commands for sending requests
- Example responses

### 2. Simple Rust Client (`simple_client.rs`)

This example shows how to create a Rust client that connects to IntelliRouter and sends a chat completion request.

**To run:**

```bash
# Make sure IntelliRouter is running in another terminal:
# intellirouter run --role router

# Run the example
cargo run --example simple_client
```

**Expected output:**
- Connection to IntelliRouter
- Sending a request asking about the capital of France
- Receiving and displaying the response

**Dependencies:**
This example requires the following dependencies in your Cargo.toml:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

### 3. Configuration Example (`config/simple.toml`)

This is an example configuration file for IntelliRouter with detailed comments explaining each setting.

**To use:**

```bash
# Copy the example config to your config directory
cp examples/config/simple.toml config/local.toml

# Start IntelliRouter with this config
intellirouter run --config config/local.toml
```

## Troubleshooting

### Common Issues

1. **IntelliRouter not starting:**
   - Check that you have the correct command: `intellirouter run`
   - Verify that the configuration file exists and is valid
   - Check for any error messages in the output

2. **Connection refused errors:**
   - Make sure IntelliRouter is running
   - Check that you're using the correct host and port
   - Verify that there are no firewall issues

3. **Authentication errors:**
   - Ensure that you've set the required API keys in your environment variables
   - Check that the API keys are valid and have not expired

4. **Missing dependencies for Rust examples:**
   - Run `cargo check` to see what dependencies are missing
   - Add the required dependencies to your Cargo.toml file

### Getting Help

If you encounter any issues not covered here, please:

1. Check the main [README.md](../README.md) for more information
2. Look for error messages in the IntelliRouter logs
3. Open an issue on the GitHub repository

## Next Steps

After exploring these basic examples, you might want to:

1. Try more advanced features like custom routing strategies
2. Explore the different roles (router, orchestrator, rag-injector, summarizer)
3. Integrate IntelliRouter with your own applications using the provided SDKs
4. Create custom plugins to extend IntelliRouter's functionality

For more information, see the [documentation](../docs/) directory.
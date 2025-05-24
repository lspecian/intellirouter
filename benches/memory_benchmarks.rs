//! Performance benchmarks for the memory module

use criterion::{criterion_group, criterion_main, Criterion};
use intellirouter::modules::memory::{Conversation, InMemoryBackend, MemoryManager, Message};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

mod framework;
use framework::{run_benchmark, BenchmarkConfig, BenchmarkType, Benchmarkable, ResourceUsage};

// Helper function to create test messages
fn create_test_messages(num_messages: usize) -> Vec<Message> {
    let mut messages = Vec::with_capacity(num_messages);

    for i in 0..num_messages {
        let role = if i % 2 == 0 { "user" } else { "assistant" };
        let content = format!("This is test message content for message {}", i);

        let mut message = Message::new(role, &content);
        message
            .metadata
            .insert("source".to_string(), format!("test_source_{}", i % 5));

        messages.push(message);
    }

    messages
}

/// Benchmark for memory manager creation
struct MemoryManagerCreationBenchmark {
    window_sizes: Vec<usize>,
}

impl MemoryManagerCreationBenchmark {
    fn new(window_sizes: Vec<usize>) -> Self {
        Self { window_sizes }
    }
}

impl Benchmarkable for MemoryManagerCreationBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Create memory managers with different window sizes
        for &window_size in &self.window_sizes {
            let backend = Arc::new(InMemoryBackend::new());
            let _manager = MemoryManager::new(backend, window_size);
        }

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "memory_manager_creation".to_string(),
            description: "Benchmark for creating memory managers".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "managers".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for adding messages to a conversation
struct MemoryAddMessageBenchmark {
    manager: Arc<MemoryManager>,
    conversation_id: String,
    messages: Vec<(String, String)>, // (role, content)
}

impl MemoryAddMessageBenchmark {
    fn new(num_messages: usize) -> Self {
        let backend = Arc::new(InMemoryBackend::new());
        let manager = Arc::new(MemoryManager::new(backend, 1000)); // Large window size

        // Create a conversation
        let rt = tokio::runtime::Runtime::new().unwrap();
        let conversation = rt.block_on(async { manager.create_conversation().await.unwrap() });
        let conversation_id = conversation.id;

        // Create messages
        let messages: Vec<(String, String)> = (0..num_messages)
            .map(|i| {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                let content = format!("This is test message content for message {}", i);
                (role.to_string(), content)
            })
            .collect();

        Self {
            manager,
            conversation_id,
            messages,
        }
    }
}

impl Benchmarkable for MemoryAddMessageBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Add all messages
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            for (role, content) in &self.messages {
                self.manager
                    .add_message(&self.conversation_id, role, content)
                    .await?;
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(start.elapsed())
    }

    fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear the conversation before each iteration
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            // Delete and recreate the conversation
            let _ = self
                .manager
                .delete_conversation(&self.conversation_id)
                .await;
            let conversation = self.manager.create_conversation().await?;

            // Update the conversation ID (this is a bit hacky since we can't modify self.conversation_id directly)
            let conversation_id = conversation.id.clone();
            unsafe {
                let ptr = &self.conversation_id as *const String as *mut String;
                *ptr = conversation_id;
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "memory_add_message".to_string(),
            description: "Benchmark for adding messages to memory".to_string(),
            benchmark_type: BenchmarkType::Throughput,
            unit: "messages".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Benchmark for retrieving messages from memory
struct MemoryGetMessagesBenchmark {
    manager: Arc<MemoryManager>,
    conversation_id: String,
    message_counts: Vec<usize>,
}

impl MemoryGetMessagesBenchmark {
    fn new(total_messages: usize, message_counts: Vec<usize>) -> Self {
        let backend = Arc::new(InMemoryBackend::new());
        let manager = Arc::new(MemoryManager::new(backend, 1000)); // Large window size

        // Create a conversation and add messages
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let conversation = manager.create_conversation().await.unwrap();
            let conversation_id = conversation.id.clone();

            // Add messages
            for i in 0..total_messages {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                let content = format!("This is test message content for message {}", i);
                manager
                    .add_message(&conversation_id, role, &content)
                    .await
                    .unwrap();
            }

            (conversation_id, manager)
        });

        let conversation_id = rt.block_on(async {
            let conversation = manager.create_conversation().await.unwrap();
            let conversation_id = conversation.id.clone();

            // Add messages
            for i in 0..total_messages {
                let role = if i % 2 == 0 { "user" } else { "assistant" };
                let content = format!("This is test message content for message {}", i);
                manager
                    .add_message(&conversation_id, role, &content)
                    .await
                    .unwrap();
            }

            conversation_id
        });

        Self {
            manager,
            conversation_id,
            message_counts,
        }
    }
}

impl Benchmarkable for MemoryGetMessagesBenchmark {
    fn run_iteration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Get messages with different counts
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            for &count in &self.message_counts {
                if count == 0 {
                    // Get all messages
                    let _ = self.manager.get_messages(&self.conversation_id).await?;
                } else {
                    // Get last N messages
                    let _ = self
                        .manager
                        .get_last_messages(&self.conversation_id, count)
                        .await?;
                }
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(start.elapsed())
    }

    fn config(&self) -> BenchmarkConfig {
        BenchmarkConfig {
            name: "memory_get_messages".to_string(),
            description: "Benchmark for retrieving messages from memory".to_string(),
            benchmark_type: BenchmarkType::Latency,
            unit: "retrievals".to_string(),
            sample_size: Some(10),
            warm_up_time: Some(Duration::from_secs(1)),
            measurement_time: Some(Duration::from_secs(5)),
        }
    }
}

fn bench_memory(c: &mut Criterion) {
    // Memory manager creation benchmark
    let window_sizes = vec![10, 50, 100, 500, 1000];
    let creation_benchmark = Arc::new(MemoryManagerCreationBenchmark::new(window_sizes));
    run_benchmark(c, creation_benchmark);

    // Memory add message benchmark
    let add_message_benchmark = Arc::new(MemoryAddMessageBenchmark::new(100));
    run_benchmark(c, add_message_benchmark);

    // Memory get messages benchmark
    let message_counts = vec![0, 5, 10, 20, 50];
    let get_messages_benchmark = Arc::new(MemoryGetMessagesBenchmark::new(100, message_counts));
    run_benchmark(c, get_messages_benchmark);
}

criterion_group!(benches, bench_memory);
criterion_main!(benches);

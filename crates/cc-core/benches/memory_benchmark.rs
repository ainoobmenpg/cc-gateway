//! Memory and Data Structure Benchmarks
//!
//! Measures performance of memory operations and data structures:
//! - Tool manager registration and lookup
//! - Message content serialization
//! - JSON parsing performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;

use cc_core::llm::{Message, MessageContent, MessagesRequest, MessagesRequestBuilder};
use cc_core::tool::{Tool, ToolManager, ToolResult};
use async_trait::async_trait;

/// Mock tool for benchmarking
struct BenchmarkTool;

#[async_trait]
impl Tool for BenchmarkTool {
    fn name(&self) -> &str {
        "benchmark_tool"
    }

    fn description(&self) -> &str {
        "A tool for benchmarking"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            }
        })
    }

    async fn execute(&self, _input: serde_json::Value) -> cc_core::Result<ToolResult> {
        Ok(ToolResult::success("benchmark result"))
    }
}

/// Benchmark tool manager operations
fn bench_tool_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_manager");

    group.bench_function("register_tool", |b| {
        b.iter(|| {
            let mut manager = ToolManager::new();
            manager.register(std::sync::Arc::new(BenchmarkTool));
            black_box(manager)
        })
    });

    group.bench_function("lookup_tool", |b| {
        let mut manager = ToolManager::new();
        manager.register(std::sync::Arc::new(BenchmarkTool));
        b.iter(|| manager.get("benchmark_tool"))
    });

    // Test registration of multiple tools
    for count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("register_multiple", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut manager = ToolManager::new();
                    for i in 0..count {
                        let tool = BenchmarkToolWithSuffix(i);
                        manager.register(std::sync::Arc::new(tool));
                    }
                    black_box(manager)
                })
            },
        );
    }

    group.finish();
}

/// Tool with suffix for unique names
struct BenchmarkToolWithSuffix(usize);

#[async_trait]
impl Tool for BenchmarkToolWithSuffix {
    fn name(&self) -> &str {
        // Use a leaked string to avoid allocation issues in benchmarks
        Box::leak(format!("benchmark_tool_{}", self.0).into_boxed_str())
    }

    fn description(&self) -> &str {
        "A tool for benchmarking"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({ "type": "object" })
    }

    async fn execute(&self, _input: serde_json::Value) -> cc_core::Result<ToolResult> {
        Ok(ToolResult::success("result"))
    }
}

/// Benchmark message operations
fn bench_message_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_operations");

    // Message creation
    group.bench_function("create_user_message", |b| {
        b.iter(|| Message::user("Hello, world!"))
    });

    // Message serialization
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("serialize_message", size), size, |b, &size| {
            let text = "x".repeat(size);
            let msg = Message::user(&text);
            b.iter(|| serde_json::to_string(black_box(&msg)).unwrap())
        });
    }

    // Message deserialization
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("deserialize_message", size),
            size,
            |b, &size| {
                let text = "x".repeat(size);
                let msg = Message::user(&text);
                let json = serde_json::to_string(&msg).unwrap();
                b.iter(|| {
                    let parsed: Message = serde_json::from_str(black_box(&json)).unwrap();
                    parsed
                })
            },
        );
    }

    group.finish();
}

/// Benchmark JSON operations
fn bench_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_operations");

    // MessagesRequest serialization
    group.bench_function("serialize_messages_request", |b| {
        let request = MessagesRequestBuilder::new("claude-sonnet-4-20250514".to_string())
            .system("You are a helpful assistant.")
            .user("Hello!")
            .user("How are you?")
            .build();

        b.iter(|| serde_json::to_string(black_box(&request)).unwrap())
    });

    // Large JSON parsing
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("parse_large_messages", count), count, |b, &count| {
            let mut builder = MessagesRequestBuilder::new("claude-sonnet-4-20250514".to_string());
            for i in 0..count {
                builder = builder.user(format!("Message {}", i));
            }
            let request = builder.build();
            let json = serde_json::to_string(&request).unwrap();

            b.iter(|| {
                let parsed: MessagesRequest = serde_json::from_str(black_box(&json)).unwrap();
                parsed
            })
        });
    }

    group.finish();
}

/// Benchmark MessageContent operations
fn bench_message_content(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_content");

    group.bench_function("create_text_content", |b| {
        b.iter(|| MessageContent::Text { text: "Hello".to_string() })
    });

    group.bench_function("serialize_text_content", |b| {
        let content = MessageContent::Text { text: "Hello".to_string() };
        b.iter(|| serde_json::to_string(black_box(&content)).unwrap())
    });

    group.bench_function("serialize_thinking_content", |b| {
        let content = MessageContent::Thinking {
            thinking: "Let me think about this...".to_string(),
            signature: Some("sig123".to_string()),
        };
        b.iter(|| serde_json::to_string(black_box(&content)).unwrap())
    });

    group.bench_function("serialize_tool_use", |b| {
        let content = MessageContent::ToolUse {
            id: "tool-123".to_string(),
            name: "read_file".to_string(),
            input: json!({ "path": "/test/file.txt" }),
        };
        b.iter(|| serde_json::to_string(black_box(&content)).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tool_manager,
    bench_message_operations,
    bench_json_operations,
    bench_message_content,
);

criterion_main!(benches);

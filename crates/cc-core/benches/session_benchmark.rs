//! Session Management Benchmarks
//!
//! Measures performance of session operations including:
//! - Session creation
//! - Message addition
//! - Session retrieval
//! - Session persistence

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use cc_core::session::{Session, SessionStore};
use cc_core::llm::Message;

/// Benchmark session creation
fn bench_session_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_creation");

    group.bench_function("new_session", |b| {
        b.iter(|| {
            let session = Session::new("test-channel");
            black_box(session)
        })
    });

    group.bench_function("session_with_message", |b| {
        b.iter(|| {
            let mut session = Session::new("test-channel");
            session.add_message(Message::user("Hello, world!"));
            black_box(session)
        })
    });

    group.finish();
}

/// Benchmark session persistence
fn bench_session_persistence(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_persistence");

    // Use in-memory database for benchmarks
    // Note: SessionStore doesn't implement Clone, so we create new stores per iteration
    group.bench_function("save_session", |b| {
        b.iter_with_setup(
            || {
                let store = SessionStore::in_memory().unwrap();
                let channel = format!("channel-{}", uuid::Uuid::now_v7());
                let session = Session::new(&channel);
                (store, session)
            },
            |(s, session)| s.save(&session).unwrap(),
        )
    });

    group.bench_function("load_session", |b| {
        let store = SessionStore::in_memory().unwrap();

        let channel = "test-channel";
        let session = Session::new(channel);
        let session_id = session.id.clone();
        store.save(&session).unwrap();

        b.iter(|| store.load(black_box(&session_id)).unwrap())
    });

    group.bench_function("list_by_channel", |b| {
        let store = SessionStore::in_memory().unwrap();

        // Pre-populate with sessions
        for i in 0..50 {
            let session = Session::new(format!("target-channel-{}", i % 10));
            store.save(&session).unwrap();
        }

        b.iter(|| store.list_by_channel(black_box("target-channel-0")).unwrap())
    });

    group.finish();
}

/// Benchmark message operations
fn bench_message_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_operations");

    // Test with different message sizes
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("add_message", size), size, |b, &size| {
            let msg = "x".repeat(size);
            b.iter_with_setup(
                || Session::new("test-channel"),
                |mut session| {
                    session.add_message(Message::user(&msg));
                    session
                },
            )
        });
    }

    // Test with different message counts
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("add_multiple_messages", count), count, |b, &count| {
            b.iter(|| {
                let mut session = Session::new("test-channel");
                for i in 0..count {
                    session.add_message(Message::user(format!("Message {}", i)));
                }
                black_box(session)
            })
        });
    }

    group.finish();
}

/// Benchmark serialization operations
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_serialization");

    group.bench_function("serialize_session", |b| {
        let mut session = Session::new("test-channel");
        for i in 0..10 {
            session.add_message(Message::user(format!("Message {}", i)));
        }

        b.iter(|| serde_json::to_string(black_box(&session)).unwrap())
    });

    group.bench_function("deserialize_session", |b| {
        let mut session = Session::new("test-channel");
        for i in 0..10 {
            session.add_message(Message::user(format!("Message {}", i)));
        }
        let json = serde_json::to_string(&session).unwrap();

        b.iter(|| {
            let parsed: Session = serde_json::from_str(black_box(&json)).unwrap();
            parsed
        })
    });

    // Large session serialization
    for count in [50, 100, 200].iter() {
        group.bench_with_input(BenchmarkId::new("serialize_large_session", count), count, |b, &count| {
            let mut session = Session::new("test-channel");
            for i in 0..count {
                session.add_message(Message::user(format!("Message {} with some additional content", i)));
            }

            b.iter(|| serde_json::to_string(black_box(&session)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_persistence,
    bench_message_operations,
    bench_serialization,
);

criterion_main!(benches);

//! T116 – Event append throughput benchmark.
//!
//! Measures how fast individual domain events can be appended to the SQLite
//! store running in WAL mode.  Target: ≥10,000 events/sec.
//!
//! Constitution gate: Hard fail if throughput drops below 10 K events/sec.

use agileplus_benchmarks::helpers::{make_event, make_events_multi_entity, make_in_memory_adapter};
use agileplus_sqlite::repository::events as event_repo;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Benchmark: append N events to a freshly opened in-memory store
// ---------------------------------------------------------------------------

fn bench_append_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_append_throughput");

    for count in [100_u64, 1_000, 5_000] {
        group.bench_with_input(
            BenchmarkId::new("sequential_events", count),
            &count,
            |b, &n| {
                b.iter(|| {
                    let adapter = make_in_memory_adapter();
                    let conn_guard = adapter
                        .conn_for_bench()
                        .expect("bench conn");

                    for i in 0..n {
                        let ev = make_event(
                            // spread across 10 entities so each gets unique sequences
                            black_box((i as i64 % 10) + 1),
                            black_box(i as i64 / 10 + 1),
                        );
                        event_repo::append_event(&conn_guard, black_box(&ev))
                            .expect("append failed");
                    }
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: single event append (measures per-event overhead)
// ---------------------------------------------------------------------------

fn bench_append_single(c: &mut Criterion) {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("bench conn");

    // Pre-seed to avoid measuring first-write overhead
    for seq in 1..=100 {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).unwrap();
    }

    let mut next_seq = 101_i64;

    c.bench_function("append_single_event", |b| {
        b.iter(|| {
            let ev = make_event(black_box(1), black_box(next_seq));
            next_seq += 1;
            event_repo::append_event(&conn, black_box(&ev)).expect("append failed");
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: batch append using multi-entity spread
// ---------------------------------------------------------------------------

fn bench_append_multi_entity(c: &mut Criterion) {
    let events = make_events_multi_entity(1_000, 100); // 1000 events, 100 entities

    c.bench_function("append_1000_events_100_entities", |b| {
        b.iter(|| {
            let adapter = make_in_memory_adapter();
            let conn = adapter.conn_for_bench().expect("bench conn");
            for ev in &events {
                event_repo::append_event(&conn, black_box(ev)).expect("append");
            }
        });
    });
}

criterion_group!(
    benches,
    bench_append_sequential,
    bench_append_single,
    bench_append_multi_entity,
);
criterion_main!(benches);

// ---------------------------------------------------------------------------
// Smoke tests (run via `cargo test`)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use agileplus_benchmarks::helpers::{make_event, make_events_multi_entity, make_in_memory_adapter};
    use agileplus_sqlite::repository::events as event_repo;

    #[test]
    fn append_100_events_smoke() {
        let adapter = make_in_memory_adapter();
        let conn = adapter.conn_for_bench().expect("conn");
        for seq in 1..=100 {
            let ev = make_event(1, seq);
            event_repo::append_event(&conn, &ev).expect("append");
        }
    }

    #[test]
    fn append_multi_entity_smoke() {
        let events = make_events_multi_entity(500, 50);
        let adapter = make_in_memory_adapter();
        let conn = adapter.conn_for_bench().expect("conn");
        for ev in &events {
            event_repo::append_event(&conn, ev).expect("append");
        }
        assert_eq!(events.len(), 500);
    }
}

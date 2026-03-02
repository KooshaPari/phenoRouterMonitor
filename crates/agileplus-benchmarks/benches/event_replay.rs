//! T117 – Event replay and snapshot rebuild benchmark.
//!
//! Measures:
//! - Full replay of 1000 events from sequence 0 (target <100ms)
//! - Snapshot-assisted replay: load snapshot at seq 900, replay 100 deltas (target <10ms)

use agileplus_benchmarks::helpers::{
    CountingAggregate, make_event, make_in_memory_adapter, make_snapshot,
};
use agileplus_events::{replay_events, replay_events_since};
use agileplus_sqlite::repository::events as event_repo;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Setup helpers (synchronous – we drive the async runtime manually)
// ---------------------------------------------------------------------------

/// Append `count` events for entity_id=1 and return the connection.
fn seed_events(count: i64) -> agileplus_sqlite::SqliteStorageAdapter {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    for seq in 1..=count {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("seed append");
    }
    drop(conn); // release the guard
    adapter
}

// ---------------------------------------------------------------------------
// Benchmark: full replay 1000 events
// ---------------------------------------------------------------------------

fn bench_full_replay_1000(c: &mut Criterion) {
    let adapter = seed_events(1000);
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("full_replay_1000_events", |b| {
        b.iter(|| {
            let conn = adapter.conn_for_bench().expect("conn");
            let events = event_repo::get_events(&conn, "Feature", black_box(1))
                .expect("get_events");
            drop(conn);

            let mut agg = CountingAggregate::default();
            rt.block_on(async {
                replay_events(&mut agg, black_box(&events))
                    .await
                    .expect("replay");
            });
            assert_eq!(agg.events_applied, 1000);
            black_box(agg.version)
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: snapshot-assisted replay (900 snapshot + 100 delta events)
// ---------------------------------------------------------------------------

fn bench_snapshot_replay_900_plus_100(c: &mut Criterion) {
    let adapter = seed_events(1000);
    // Simulate: snapshot at seq 900, only events 901..1000 need replaying
    let snapshot = make_snapshot(1, 900);
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("snapshot_replay_900_plus_100", |b| {
        b.iter(|| {
            // Load delta events since snapshot sequence
            let conn = adapter.conn_for_bench().expect("conn");
            let delta_events =
                event_repo::get_events_since(&conn, "Feature", 1, black_box(snapshot.event_sequence))
                    .expect("get_events_since");
            drop(conn);

            // Reconstruct from snapshot state (pre-built aggregate version)
            let mut agg = CountingAggregate {
                version: snapshot.event_sequence,
                events_applied: snapshot.event_sequence as u64,
                last_state: "Specified".to_string(),
            };

            rt.block_on(async {
                replay_events_since(&mut agg, black_box(snapshot.event_sequence), &delta_events)
                    .await
                    .expect("replay_since");
            });

            // After replaying 100 delta events the aggregate should be at seq 1000
            assert_eq!(agg.version, 1000);
            black_box(agg.events_applied)
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: baseline event-application rate (pure in-memory, no DB)
// ---------------------------------------------------------------------------

fn bench_event_application_rate(c: &mut Criterion) {
    let events: Vec<_> = (1..=1000).map(|seq| make_event(1, seq)).collect();
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("apply_1000_events_in_memory", |b| {
        b.iter(|| {
            let mut agg = CountingAggregate::default();
            rt.block_on(async {
                replay_events(&mut agg, black_box(&events))
                    .await
                    .expect("replay");
            });
            black_box(agg.events_applied)
        });
    });
}

criterion_group!(
    benches,
    bench_full_replay_1000,
    bench_snapshot_replay_900_plus_100,
    bench_event_application_rate,
);
criterion_main!(benches);

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn full_replay_smoke() {
        let adapter = seed_events(100);
        let conn = adapter.conn_for_bench().expect("conn");
        let events = event_repo::get_events(&conn, "Feature", 1).expect("get");
        drop(conn);
        assert_eq!(events.len(), 100);

        let mut agg = CountingAggregate::default();
        replay_events(&mut agg, &events).await.expect("replay");
        assert_eq!(agg.events_applied, 100);
        assert_eq!(agg.version, 100);
    }

    #[tokio::test]
    async fn snapshot_replay_smoke() {
        let adapter = seed_events(200);
        let snapshot = make_snapshot(1, 100);

        let conn = adapter.conn_for_bench().expect("conn");
        let delta = event_repo::get_events_since(&conn, "Feature", 1, snapshot.event_sequence)
            .expect("get_since");
        drop(conn);
        assert_eq!(delta.len(), 100);

        let mut agg = CountingAggregate {
            version: 100,
            events_applied: 100,
            last_state: "Specified".to_string(),
        };
        replay_events_since(&mut agg, snapshot.event_sequence, &delta)
            .await
            .expect("replay_since");
        assert_eq!(agg.version, 200);
    }
}

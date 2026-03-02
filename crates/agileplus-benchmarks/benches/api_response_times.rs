//! T118 – API response time benchmarks.
//!
//! Rather than spinning up a full HTTP server for each iteration (which would
//! measure networking overhead, not handler logic), we call the domain-layer
//! operations that the API handlers delegate to.  This gives us accurate
//! measurements of the handler hot-path latency excluding I/O scheduling.
//!
//! SLO targets (constitution gates, warning level):
//! - List features (100 items):    p95 < 100 ms
//! - Get feature by ID:            p95 < 50 ms
//! - Feature state transition:     p95 < 100 ms
//! - Health-check equivalent:       p95 < 10 ms

use agileplus_benchmarks::helpers::{make_feature, make_in_memory_adapter};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_sqlite::SqliteStorageAdapter;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Setup helpers
// ---------------------------------------------------------------------------

/// Seed `n` features and return the adapter.
fn seed_features(n: i64) -> SqliteStorageAdapter {
    use agileplus_sqlite::repository::features as feat_repo;
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    for i in 1..=n {
        let f = make_feature(i);
        feat_repo::create_feature(&conn, &f).expect("create");
    }
    drop(conn);
    adapter
}

// ---------------------------------------------------------------------------
// Benchmark: list all features (simulates GET /api/features)
// ---------------------------------------------------------------------------

fn bench_list_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_list_features");

    for count in [10_i64, 100] {
        let adapter = seed_features(count);
        group.bench_with_input(
            BenchmarkId::new("list_features", count),
            &count,
            |b, _| {
                use agileplus_sqlite::repository::features as feat_repo;
                b.iter(|| {
                    let conn = adapter.conn_for_bench().expect("conn");
                    let features = feat_repo::list_all_features(&conn).expect("list");
                    black_box(features.len())
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: get feature by ID (simulates GET /api/features/:id)
// ---------------------------------------------------------------------------

fn bench_get_feature_by_id(c: &mut Criterion) {
    use agileplus_sqlite::repository::features as feat_repo;
    let adapter = seed_features(100);

    c.bench_function("get_feature_by_id", |b| {
        b.iter(|| {
            let conn = adapter.conn_for_bench().expect("conn");
            let f = feat_repo::get_feature_by_id(&conn, black_box(50)).expect("get");
            black_box(f.map(|x| x.id))
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: feature state transition (simulates POST /api/features/:id/transition)
// ---------------------------------------------------------------------------

fn bench_feature_transition(c: &mut Criterion) {
    use agileplus_sqlite::repository::features as feat_repo;
    let adapter = seed_features(1);

    c.bench_function("feature_state_transition", |b| {
        b.iter(|| {
            // Reset to Created then advance to Specified (allowed transition).
            let conn = adapter.conn_for_bench().expect("conn");
            feat_repo::update_feature_state(&conn, black_box(1), black_box(FeatureState::Specified))
                .expect("transition");
            // Reset for next iteration
            feat_repo::update_feature_state(&conn, 1, FeatureState::Created)
                .expect("reset");
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: health-check equivalent (DB ping)
// ---------------------------------------------------------------------------

fn bench_health_check(c: &mut Criterion) {
    let adapter = make_in_memory_adapter();

    c.bench_function("health_check_db_ping", |b| {
        b.iter(|| {
            let conn = adapter.conn_for_bench().expect("conn");
            // Simple query mimicking a health-check liveness probe
            let result: i64 = conn
                .query_row("SELECT 1", [], |row| row.get(0))
                .expect("ping");
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_list_features,
    bench_get_feature_by_id,
    bench_feature_transition,
    bench_health_check,
);
criterion_main!(benches);

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod tests {
    use super::seed_features;
    use agileplus_benchmarks::helpers::make_in_memory_adapter;
    use agileplus_domain::domain::state_machine::FeatureState;
    use agileplus_sqlite::repository::features as feat_repo;

    #[test]
    fn list_100_features_smoke() {
        let adapter = seed_features(100);
        let conn = adapter.conn_for_bench().expect("conn");
        let features = feat_repo::list_all_features(&conn).expect("list");
        assert_eq!(features.len(), 100);
    }

    #[test]
    fn get_feature_by_id_smoke() {
        let adapter = seed_features(10);
        let conn = adapter.conn_for_bench().expect("conn");
        let f = feat_repo::get_feature_by_id(&conn, 5).expect("get");
        assert!(f.is_some());
    }

    #[test]
    fn feature_transition_smoke() {
        let adapter = seed_features(1);
        let conn = adapter.conn_for_bench().expect("conn");
        feat_repo::update_feature_state(&conn, 1, FeatureState::Specified)
            .expect("transition");
        let f = feat_repo::get_feature_by_id(&conn, 1).expect("get").unwrap();
        assert_eq!(f.state, FeatureState::Specified);
    }

    #[test]
    fn health_check_smoke() {
        let adapter = make_in_memory_adapter();
        let conn = adapter.conn_for_bench().expect("conn");
        let result: i64 = conn.query_row("SELECT 1", [], |row| row.get(0)).expect("ping");
        assert_eq!(result, 1);
    }
}

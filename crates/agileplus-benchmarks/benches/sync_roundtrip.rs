//! T119 – Sync round-trip benchmark.
//!
//! Benchmarks the serialise → transmit → deserialise hot-path for Plane.so sync
//! operations.  A live network is not required: we use an in-process mock that
//! measures the serialisation cost plus the domain-layer state-mapping logic.
//!
//! Constitution gates (warning level):
//! - Push / pull single feature:   < 2 s
//! - Full sync 100 features:       < 30 s
//! - Sync with 5 conflicts:        < 10 s

use agileplus_benchmarks::helpers::{
    SyncPayload, make_sync_payloads, simulate_sync_roundtrip,
};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Benchmark: single push (serialise + map to Plane schema)
// ---------------------------------------------------------------------------

fn bench_push_single(c: &mut Criterion) {
    let payload = SyncPayload::new(1);

    c.bench_function("push_single_feature", |b| {
        b.iter(|| {
            // Simulate outbound mapping: domain → Plane.so JSON
            let json = serde_json::json!({
                "title":       black_box(&payload.slug),
                "state":       format!("{:?}", black_box(payload.state)),
                "description": black_box(&payload.description),
            });
            black_box(json.to_string().len())
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: single pull (deserialise + map from Plane schema)
// ---------------------------------------------------------------------------

fn bench_pull_single(c: &mut Criterion) {
    let raw_json = serde_json::json!({
        "id":          1_i64,
        "slug":        "feature-1",
        "state":       "Specified",
        "description": "x".repeat(256),
    })
    .to_string();

    c.bench_function("pull_single_feature", |b| {
        b.iter(|| {
            // Simulate inbound mapping: Plane.so JSON → domain
            let v: serde_json::Value =
                serde_json::from_str(black_box(&raw_json)).expect("deser");
            black_box(v["id"].as_i64().unwrap())
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: full round-trip for 1, 10, 100 features
// ---------------------------------------------------------------------------

fn bench_full_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_roundtrip_n_features");

    for count in [1_i64, 10, 100] {
        let payloads = make_sync_payloads(count);
        group.bench_with_input(
            BenchmarkId::new("sync_n_features_no_conflicts", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(payloads.len());
                    for p in &payloads {
                        results.push(simulate_sync_roundtrip(black_box(p)));
                    }
                    black_box(results.len())
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: sync with conflict injection (5 conflicts out of 100)
// ---------------------------------------------------------------------------

fn bench_sync_with_conflicts(c: &mut Criterion) {
    let mut payloads = make_sync_payloads(100);

    // Inject conflicts: mark 5 features as having a diverged description
    for i in [10_usize, 20, 30, 40, 50] {
        payloads[i].description = format!("CONFLICT-{i}-{}", "x".repeat(512));
    }

    c.bench_function("sync_100_features_5_conflicts", |b| {
        b.iter(|| {
            let mut resolved = 0_u64;
            for p in &payloads {
                let out = simulate_sync_roundtrip(black_box(p));
                if out.description.starts_with("CONFLICT") {
                    // Simulate conflict resolution: pick local (no-op in mock)
                    resolved += 1;
                }
                black_box(&out);
            }
            black_box(resolved)
        });
    });
}

criterion_group!(
    benches,
    bench_push_single,
    bench_pull_single,
    bench_full_sync,
    bench_sync_with_conflicts,
);
criterion_main!(benches);

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use agileplus_benchmarks::helpers::{SyncPayload, make_sync_payloads, simulate_sync_roundtrip};

    #[test]
    fn push_single_smoke() {
        let p = SyncPayload::new(1);
        let json = serde_json::json!({
            "title":       &p.slug,
            "state":       format!("{:?}", p.state),
            "description": &p.description,
        });
        assert!(!json.to_string().is_empty());
    }

    #[test]
    fn pull_single_smoke() {
        let raw = serde_json::json!({
            "id": 1_i64,
            "slug": "feature-1",
            "state": "Specified",
            "description": "hello",
        })
        .to_string();
        let v: serde_json::Value = serde_json::from_str(&raw).expect("deser");
        assert_eq!(v["id"].as_i64(), Some(1));
    }

    #[test]
    fn full_sync_100_smoke() {
        let payloads = make_sync_payloads(100);
        let results: Vec<_> = payloads.iter().map(simulate_sync_roundtrip).collect();
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn conflict_resolution_smoke() {
        let mut payloads = make_sync_payloads(10);
        payloads[3].description = "CONFLICT-3".to_string();
        let mut conflicts = 0;
        for p in &payloads {
            let out = simulate_sync_roundtrip(p);
            if out.description.starts_with("CONFLICT") {
                conflicts += 1;
            }
        }
        assert_eq!(conflicts, 1);
    }
}

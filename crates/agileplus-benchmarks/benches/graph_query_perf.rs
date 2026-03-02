//! T120 – Graph query performance benchmark.
//!
//! Benchmarks the in-memory `GraphBackend` to measure the overhead of:
//! - Node creation (Feature, WorkPackage, Agent)
//! - Node lookup by ID
//! - Dependency / blocking-path traversal queries
//! - Bulk node creation (seeding 100 features)
//!
//! Note: The in-memory backend doesn't parse Cypher; it pattern-matches
//! query strings.  These benchmarks therefore measure the overhead of the
//! Rust-side dispatch layer rather than a real graph database.
//! A Neo4j-backed benchmark would be added in CI using the `neo4j` feature.

use agileplus_benchmarks::helpers::make_feature;
use agileplus_graph::{
    GraphConfig, GraphStore, NodeStore, RelationshipStore, GraphQueries,
};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use tokio::runtime::Runtime;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_store() -> GraphStore {
    GraphStore::in_memory(GraphConfig::default())
}

/// Seed N feature nodes into the store and return it.
async fn seed_features(store: &GraphStore, n: i64) {
    let nodes = NodeStore::new(store);
    for i in 1..=n {
        let f = make_feature(i);
        nodes
            .create_feature(f.id, f.slug.clone(), format!("{:?}", f.state), f.friendly_name.clone())
            .await
            .expect("create feature node");
    }
}

// ---------------------------------------------------------------------------
// Benchmark: create a single feature node
// ---------------------------------------------------------------------------

fn bench_create_feature_node(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("graph_create_feature_node", |b| {
        b.iter(|| {
            let store = make_store();
            let nodes = NodeStore::new(&store);
            rt.block_on(async {
                nodes
                    .create_feature(
                        black_box(1),
                        black_box("feat-bench".to_string()),
                        black_box("Created".to_string()),
                        black_box("Bench Feature".to_string()),
                    )
                    .await
                    .expect("create");
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: get feature node by ID
// ---------------------------------------------------------------------------

fn bench_get_feature_node(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let store = make_store();
    rt.block_on(seed_features(&store, 100));

    c.bench_function("graph_get_feature_node", |b| {
        b.iter(|| {
            let nodes = NodeStore::new(&store);
            rt.block_on(async {
                let f = nodes.get_feature(black_box(50)).await.expect("get");
                black_box(f.map(|v| v["id"].as_i64()))
            })
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: seed N nodes (measures bulk-creation throughput)
// ---------------------------------------------------------------------------

fn bench_seed_n_features(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("graph_seed_features");

    for count in [10_i64, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("seed_features", count),
            &count,
            |b, &n| {
                b.iter(|| {
                    let store = make_store();
                    rt.block_on(seed_features(&store, black_box(n)));
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: relationship creation
// ---------------------------------------------------------------------------

fn bench_create_relationships(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let store = make_store();
    rt.block_on(async {
        // Seed features 1..=10 and WPs 1..=10
        let nodes = NodeStore::new(&store);
        for i in 1..=10_i64 {
            nodes.create_feature(i, format!("f-{i}"), "Created".into(), format!("F{i}")).await.unwrap();
            nodes.create_workpackage(i, format!("WP-{i}"), "todo".into(), i as i32).await.unwrap();
        }
    });

    c.bench_function("graph_create_owns_relationship", |b| {
        b.iter(|| {
            let rels = RelationshipStore::new(&store);
            rt.block_on(async {
                rels.owns(black_box(1), black_box(1))
                    .await
                    .expect("owns");
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: dependency-chain traversal query
// ---------------------------------------------------------------------------

fn bench_dependency_chain_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let store = make_store();
    rt.block_on(seed_features(&store, 100));

    c.bench_function("graph_dependency_chain_query", |b| {
        b.iter(|| {
            let queries = GraphQueries::new(&store);
            rt.block_on(async {
                let chain = queries.get_dependency_chain(black_box(1)).await.expect("query");
                black_box(chain.len())
            })
        });
    });
}

criterion_group!(
    benches,
    bench_create_feature_node,
    bench_get_feature_node,
    bench_seed_n_features,
    bench_create_relationships,
    bench_dependency_chain_query,
);
criterion_main!(benches);

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::{make_store, seed_features};
    use agileplus_graph::{NodeStore, RelationshipStore, GraphQueries};

    #[tokio::test]
    async fn create_and_get_feature_smoke() {
        let store = make_store();
        let nodes = NodeStore::new(&store);
        nodes
            .create_feature(1, "smoke".into(), "Created".into(), "Smoke".into())
            .await
            .unwrap();
        let f = nodes.get_feature(1).await.unwrap();
        assert!(f.is_some());
    }

    #[tokio::test]
    async fn seed_100_features_smoke() {
        let store = make_store();
        seed_features(&store, 100).await;
        // No panic = success; in-memory backend doesn't enforce uniqueness
        let nodes = NodeStore::new(&store);
        let f = nodes.get_feature(50).await.unwrap();
        assert!(f.is_some());
    }

    #[tokio::test]
    async fn dependency_chain_empty_smoke() {
        let store = make_store();
        seed_features(&store, 10).await;
        let queries = GraphQueries::new(&store);
        let chain = queries.get_dependency_chain(1).await.unwrap();
        // In-memory backend returns empty for traversal queries
        assert!(chain.is_empty());
    }

    #[tokio::test]
    async fn relationship_create_smoke() {
        let store = make_store();
        let nodes = NodeStore::new(&store);
        nodes.create_feature(1, "f1".into(), "Created".into(), "F1".into()).await.unwrap();
        nodes.create_workpackage(1, "WP1".into(), "todo".into(), 1).await.unwrap();
        let rels = RelationshipStore::new(&store);
        rels.owns(1, 1).await.unwrap();
    }
}

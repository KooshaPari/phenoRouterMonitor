#![allow(dead_code)]

mod observability;
mod storage;
mod storage_port_impl;
mod vcs;

use std::sync::Arc;

use agileplus_api::{AppState, create_router};
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::{CredentialStore, InMemoryCredentialStore, keys as cred_keys};
use axum_test::TestServer;

use self::observability::MockObs;
pub(crate) use self::storage::MockStorage;
use self::vcs::MockVcs;

pub(crate) const TEST_API_KEY: &str = "test-api-key-12345";

pub(crate) async fn setup_test_server() -> TestServer {
    setup_test_server_with_storage(MockStorage::with_test_data()).await
}

pub(crate) async fn setup_test_server_with_storage(storage: MockStorage) -> TestServer {
    let storage = Arc::new(storage);
    let vcs = Arc::new(MockVcs);
    let telemetry = Arc::new(MockObs);
    let config = Arc::new(AppConfig::default());

    let creds_inner = InMemoryCredentialStore::new();
    creds_inner
        .set("agileplus", cred_keys::API_KEYS, TEST_API_KEY)
        .expect("setting test API key should succeed");
    let creds: Arc<dyn CredentialStore> = Arc::new(creds_inner);

    let state = AppState::new(storage, vcs, telemetry, config, creds);
    let app = create_router(state);
    TestServer::new(app)
}

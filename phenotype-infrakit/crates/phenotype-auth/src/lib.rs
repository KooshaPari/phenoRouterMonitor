//! phenotype-auth
//!
//! Lightweight authentication primitives (ports + simple JWT/session utilities)

pub mod ports;
pub mod jwt;
pub mod session;
pub mod middleware;

pub use jwt::JwtValidator;
pub use session::SessionManager;
pub use middleware::auth_middleware;

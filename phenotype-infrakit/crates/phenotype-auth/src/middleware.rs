/// Thin middleware helper placeholder. Consumers should implement adapter-specific middleware.

pub fn auth_middleware_description() -> &'static str {
    "Call into phenotype-auth's JwtValidator and SessionManager to validate requests."
}

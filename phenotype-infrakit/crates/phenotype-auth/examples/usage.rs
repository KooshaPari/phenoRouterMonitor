fn main() {
    println!("phenotype-auth example");
    // Example usage (not a real integration):
    let desc = phenotype_auth::auth_middleware_description();
    println!("{}", desc);
}

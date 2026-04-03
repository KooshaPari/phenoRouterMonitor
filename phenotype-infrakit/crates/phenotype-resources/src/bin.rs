//! BKM-01: CLI for thegent-resources. Outputs JSON to stdout.

fn main() {
    let snapshot = thegent_resources::sample();
    let json = serde_json::to_string(&snapshot).expect("serialize");
    println!("{json}");
}

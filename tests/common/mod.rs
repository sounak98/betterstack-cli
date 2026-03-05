use std::path::PathBuf;

#[allow(dead_code)]
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/common/fixtures")
        .join(name)
}

#[allow(dead_code)]
pub fn load_fixture(name: &str) -> serde_json::Value {
    let path = fixture_path(name);
    let contents = std::fs::read_to_string(path).expect("fixture file not found");
    serde_json::from_str(&contents).expect("invalid JSON in fixture")
}

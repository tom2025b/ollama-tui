use super::super::host::normalize_host;

#[test]
fn normalize_host_adds_http_scheme_when_missing() {
    assert_eq!(
        normalize_host("127.0.0.1:11434/".to_string()),
        "http://127.0.0.1:11434"
    );
}

#[test]
fn normalize_host_keeps_existing_scheme() {
    assert_eq!(
        normalize_host("https://example.test:11434/".to_string()),
        "https://example.test:11434"
    );
}

//! Envelope parsing tests.

use mcsm_protocol::ApiEnvelope;

#[test]
fn envelope_ok() {
    let raw = r#"{"status":200,"data":{"hello":"world"},"time":1710000000000}"#;
    let env: ApiEnvelope<serde_json::Value> = serde_json::from_str(raw).unwrap();
    assert!(env.is_ok());
    let v = env.into_result_value().unwrap();
    assert_eq!(v["hello"], "world");
}

#[test]
fn envelope_err() {
    let raw = r#"{"status":500,"data":"boom","time":1}"#;
    let env: ApiEnvelope<serde_json::Value> = serde_json::from_str(raw).unwrap();
    assert!(!env.is_ok());
    let err = env.into_result_value().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("500"));
    assert!(msg.contains("boom"));
}

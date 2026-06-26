use serde_json::Value;
use std::process::Command;

#[test]
fn demo_mode_outputs_mesh_recovery_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_tendril"))
        .arg("--demo")
        .output()
        .expect("run tendril --demo");

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("demo output is JSON");

    assert_eq!(report["mesh_name"], "tendril-demo");
    assert_eq!(report["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(report["recovering"].as_array().unwrap().len(), 1);
    assert!(output.stderr.is_empty());
}

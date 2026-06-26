use serde_json::Value;
use std::fs;
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

#[test]
fn demo_report_writes_static_html() {
    let report_path =
        std::env::temp_dir().join(format!("tendril-demo-report-{}.html", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_tendril"))
        .arg("--demo-report")
        .arg(&report_path)
        .output()
        .expect("run tendril --demo-report");

    assert!(output.status.success());
    let html = fs::read_to_string(&report_path).expect("read report");

    assert!(html.contains("Tendril Demo Report"));
    assert!(html.contains("pulse-alpha"));
    assert!(html.contains("Recovering"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("report:"));

    let _ = fs::remove_file(report_path);
}

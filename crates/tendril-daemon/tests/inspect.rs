use serde_json::json;
use std::fs;
use std::process::Command;

#[test]
fn status_reports_registry_counts() {
    let registry_path = temp_registry_path("status");
    fs::write(
        &registry_path,
        serde_json::to_string_pretty(&json!([
            {
                "id": "00000000-0000-4000-8000-000000000001",
                "name": "node-a",
                "addr": "127.0.0.1:4101",
                "state": "Alive",
                "last_seen": "2026-06-26T00:00:00Z",
                "mac_addr": null
            },
            {
                "id": "00000000-0000-4000-8000-000000000002",
                "name": "node-b",
                "addr": "127.0.0.1:4102",
                "state": "Recovering",
                "last_seen": "2026-06-26T00:00:00Z",
                "mac_addr": "aa:bb:cc:dd:ee:ff"
            }
        ]))
        .unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tendril"))
        .arg("--status")
        .arg("--registry")
        .arg(&registry_path)
        .output()
        .expect("run tendril --status");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("registry nodes: 2"));
    assert!(stdout.contains("alive: 1"));
    assert!(stdout.contains("recovering: 1"));

    let _ = fs::remove_file(registry_path);
}

#[test]
fn nodes_lists_registry_entries() {
    let registry_path = temp_registry_path("nodes");
    fs::write(
        &registry_path,
        serde_json::to_string_pretty(&json!([
            {
                "id": "00000000-0000-4000-8000-000000000003",
                "name": "node-c",
                "addr": "127.0.0.1:4103",
                "state": "Dead",
                "last_seen": "2026-06-26T00:00:00Z",
                "mac_addr": null
            }
        ]))
        .unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tendril"))
        .arg("--nodes")
        .arg("--registry")
        .arg(&registry_path)
        .output()
        .expect("run tendril --nodes");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("node-c 127.0.0.1:4103 Dead mac=none"));

    let _ = fs::remove_file(registry_path);
}

fn temp_registry_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tendril-{name}-registry-{}.json",
        std::process::id()
    ))
}

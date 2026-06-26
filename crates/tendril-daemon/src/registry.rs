use anyhow::Result;
use std::path::{Path, PathBuf};
use tendril_core::node::Node;

pub fn load(path: impl AsRef<Path>) -> Result<Vec<Node>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save(path: impl AsRef<Path>, nodes: &[Node]) -> Result<PathBuf> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(nodes)?;
    std::fs::write(path, raw)?;
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{load, save};
    use tendril_core::node::Node;

    #[test]
    fn registry_round_trips_nodes() {
        let path =
            std::env::temp_dir().join(format!("tendril-registry-{}.json", std::process::id()));
        let node = Node::new("peer-a", "127.0.0.1:7777", Some("aa:bb:cc:dd:ee:ff"));

        save(&path, std::slice::from_ref(&node)).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, node.id);
        assert_eq!(loaded[0].name, "peer-a");
        assert_eq!(loaded[0].mac_addr.as_deref(), Some("aa:bb:cc:dd:ee:ff"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn missing_registry_loads_empty() {
        let path = std::env::temp_dir().join(format!(
            "missing-tendril-registry-{}.json",
            std::process::id()
        ));

        assert!(load(path).unwrap().is_empty());
    }
}

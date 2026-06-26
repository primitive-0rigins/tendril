mod config;
mod mesh;
mod recovery;
mod listener;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::load("tendril.toml")?;
    info!("Tendril daemon starting — mesh: {}", cfg.mesh_name);

    let mesh = mesh::Mesh::new(cfg.clone());

    tokio::select! {
        _ = listener::run(mesh.clone(), cfg.listen_addr.clone()) => {},
        _ = mesh::run_heartbeat_watcher(mesh.clone()) => {},
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down.");
        }
    }

    Ok(())
}

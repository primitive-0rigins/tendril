mod config;
mod demo;
mod listener;
mod mesh;
mod recovery;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut args = std::env::args().skip(1);
    if args.any(|arg| arg == "--demo") {
        return demo::run().await;
    }
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--demo-report" {
            let path = args
                .next()
                .unwrap_or_else(|| "target/tendril-demo-report.html".to_string());
            let path = demo::write_html(path).await?;
            println!("report: {}", path.display());
            return Ok(());
        }
    }

    let cfg = config::load("tendril.toml")?;
    info!(
        "Tendril daemon starting — mesh: {}, node: {}, listen: {}",
        cfg.mesh_name, cfg.node_name, cfg.listen_addr
    );

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

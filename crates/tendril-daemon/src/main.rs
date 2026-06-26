mod config;
mod demo;
mod listener;
mod mesh;
mod recovery;
mod registry;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse(std::env::args().skip(1));
    if args.demo {
        return demo::run().await;
    }
    if let Some(path) = args.demo_report {
        let path = demo::write_html(path).await?;
        println!("report: {}", path.display());
        return Ok(());
    }

    let cfg = config::load("tendril.toml")?;
    info!(
        "Tendril daemon starting — mesh: {}, node: {}, listen: {}",
        cfg.mesh_name, cfg.node_name, cfg.listen_addr
    );

    let saved_nodes = match &args.registry {
        Some(path) => registry::load(path)?,
        None => Vec::new(),
    };
    let mesh = mesh::Mesh::with_nodes(cfg.clone(), saved_nodes);

    tokio::select! {
        _ = listener::run(mesh.clone(), cfg.listen_addr.clone()) => {},
        _ = mesh::run_heartbeat_watcher(mesh.clone()) => {},
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down.");
            if let Some(path) = &args.registry {
                let nodes = mesh.node_list().await;
                registry::save(path, &nodes)?;
                info!("Saved registry: {}", path);
            }
        }
    }

    Ok(())
}

struct Args {
    demo: bool,
    demo_report: Option<String>,
    registry: Option<String>,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut parsed = Self {
            demo: false,
            demo_report: None,
            registry: None,
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--demo" => parsed.demo = true,
                "--demo-report" => {
                    parsed.demo_report = Some(
                        args.next()
                            .unwrap_or_else(|| "target/tendril-demo-report.html".into()),
                    );
                }
                "--registry" => parsed.registry = args.next(),
                _ => {}
            }
        }
        parsed
    }
}

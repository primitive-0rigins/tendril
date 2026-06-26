mod config;
mod demo;
mod listener;
mod mesh;
mod recovery;
mod registry;

use anyhow::Result;
use tendril_core::node::NodeState;
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
    if args.status {
        print_status(&cfg, args.registry.as_deref())?;
        return Ok(());
    }
    if args.nodes {
        print_nodes(args.registry.as_deref())?;
        return Ok(());
    }

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
    nodes: bool,
    registry: Option<String>,
    status: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Self {
        let mut parsed = Self {
            demo: false,
            demo_report: None,
            nodes: false,
            registry: None,
            status: false,
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
                "--nodes" => parsed.nodes = true,
                "--registry" => parsed.registry = args.next(),
                "--status" => parsed.status = true,
                _ => {}
            }
        }
        parsed
    }
}

fn print_status(cfg: &config::Config, registry_path: Option<&str>) -> Result<()> {
    let nodes = registry_path
        .map(registry::load)
        .transpose()?
        .unwrap_or_default();
    let alive = nodes
        .iter()
        .filter(|node| node.state == NodeState::Alive)
        .count();
    let recovering = nodes
        .iter()
        .filter(|node| node.state == NodeState::Recovering)
        .count();
    let dead = nodes
        .iter()
        .filter(|node| node.state == NodeState::Dead)
        .count();

    println!("mesh: {}", cfg.mesh_name);
    println!("node: {}", cfg.node_name);
    println!("listen: {}", cfg.listen_addr);
    println!("registry nodes: {}", nodes.len());
    println!("alive: {alive}");
    println!("recovering: {recovering}");
    println!("dead: {dead}");
    Ok(())
}

fn print_nodes(registry_path: Option<&str>) -> Result<()> {
    let Some(path) = registry_path else {
        println!("No registry path provided. Use --registry <path> --nodes.");
        return Ok(());
    };
    let nodes = registry::load(path)?;
    if nodes.is_empty() {
        println!("No nodes in registry.");
        return Ok(());
    }

    for node in nodes {
        let mac = node.mac_addr.as_deref().unwrap_or("none");
        println!("{} {} {:?} mac={}", node.name, node.addr, node.state, mac);
    }
    Ok(())
}

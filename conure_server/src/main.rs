use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use client_manager::ClientManager;
use config::Config;

use conure_common::conure_rpc_capnp;
use futures::AsyncReadExt;
use server::GatewayImpl;
use ui::ui_main;

mod client_manager;
mod config;
mod server;
mod ui;

async fn run_rpc_server(
    config: Config,
    client_manager: ClientManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(&config.address).await?;
    let hello_world_client: conure_rpc_capnp::gateway::Client =
        capnp_rpc::new_client(GatewayImpl::new(client_manager.clone()));

    loop {
        let (stream, _) = listener.accept().await?;
        stream.set_nodelay(true)?;
        let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
        let network = twoparty::VatNetwork::new(
            futures::io::BufReader::new(reader),
            futures::io::BufWriter::new(writer),
            rpc_twoparty_capnp::Side::Server,
            Default::default(),
        );
        let rpc_system = RpcSystem::new(Box::new(network), Some(hello_world_client.clone().client));
        tokio::task::spawn_local(rpc_system);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let config = Config::load();
    let client_manager = ClientManager::new();

    // Taskset for capnp
    let local = tokio::task::LocalSet::new();

    // Start the UI server in the background
    tokio::spawn(ui_main(client_manager.clone()));

    // Run the RPC server in the local task set
    local
        .run_until(async {
            if let Err(e) = run_rpc_server(config, client_manager).await {
                eprintln!("RPC server error: {}", e);
            }
        })
        .await;

    Ok(())

    // tokio::task::LocalSet::new()
    //     .run_until(async move {
    //         let listener = tokio::net::TcpListener::bind(&config.address).await?;
    //         let hello_world_client: conure_rpc_capnp::gateway::Client =
    //             capnp_rpc::new_client(GatewayImpl::new(client_manager.clone()));

    //         loop {
    //             let (stream, _) = listener.accept().await?;
    //             stream.set_nodelay(true)?;
    //             let (reader, writer) =
    //                 tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
    //             let network = twoparty::VatNetwork::new(
    //                 futures::io::BufReader::new(reader),
    //                 futures::io::BufWriter::new(writer),
    //                 rpc_twoparty_capnp::Side::Server,
    //                 Default::default(),
    //             );

    //             let rpc_system =
    //                 RpcSystem::new(Box::new(network), Some(hello_world_client.clone().client));

    //             tokio::task::spawn_local(rpc_system);
    //         }
    //     })
    //     .await
}

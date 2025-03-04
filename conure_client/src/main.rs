use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use conure_common::conure_rpc_capnp::{
    client_rpc,
    gateway::{self},
    server_rpc,
};
use futures::AsyncReadExt;
use rpc::ClientRpcImpl;

mod rpc;
mod system_info;

static ADDRESS: &str = "127.0.0.1:3000";

// TODO: better error handling, retry upon any issue
// GOAL: make it never stop running
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    tokio::task::LocalSet::new()
        .run_until(async move {
            let stream = tokio::net::TcpStream::connect(&ADDRESS).await?;
            stream.set_nodelay(true)?;
            let (reader, writer) =
                tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
            let rpc_network = Box::new(twoparty::VatNetwork::new(
                futures::io::BufReader::new(reader),
                futures::io::BufWriter::new(writer),
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));
            let mut rpc_system = RpcSystem::new(rpc_network, None);
            let gateway_connection: gateway::Client =
                rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

            tokio::task::spawn_local(rpc_system);

            // Client instance
            let server_rpc = rpc_exchange(
                &gateway_connection,
                capnp_rpc::new_client(ClientRpcImpl),
                "hellothere",
            )
            .await?;

            let mut report_req = server_rpc.report_system_info_request();
            let sys_info = system_info::collect_system_info()?;
            report_req
                .get()
                .set_info(sys_info.build_message().get_root_as_reader()?)?;

            loop {
                // Send updated system info
                let mut report_req = server_rpc.report_system_info_request();
                let sys_info = system_info::collect_system_info()?;
                report_req
                    .get()
                    .set_info(sys_info.build_message().get_root_as_reader()?)?;
                report_req.send().promise.await?;

                println!("Sent periodic system info update");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }

            #[allow(unreachable_code)]
            {
                // Wait for the RPC system to complete (only on error)
                report_req.send().promise.await?;
                Ok(())
            }
        })
        .await
}

/// Initialize the RPC exchange.
async fn rpc_exchange(
    gateway: &gateway::Client,
    client_rpc: client_rpc::Client,
    token: &str,
) -> Result<server_rpc::Client, capnp::Error> {
    let mut req = gateway.register_client_request();
    let mut params = req.get().init_params();

    params.set_token(token.to_string());
    params.set_client(client_rpc);

    let reply = req.send().promise.await?;
    reply.get().unwrap().get_rpc()
}

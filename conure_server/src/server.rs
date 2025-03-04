use capnp::capability::Promise;
use capnp_rpc::pry;
// use capnp_rpc::{RpcSystem, pry, rpc_twoparty_capnp, twoparty};
use conure_common::{
    conure_rpc_capnp::{gateway, server_rpc},
    system_info::SystemInfo,
};
use derive_new::new;

use crate::client_manager::{ActiveClient, ClientManager};

#[derive(new)]
pub struct ServerRpcImpl {
    client: ActiveClient,
}

impl server_rpc::Server for ServerRpcImpl {
    fn report_system_info(
        &mut self,
        params: server_rpc::ReportSystemInfoParams,
        mut _results: server_rpc::ReportSystemInfoResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let request = pry!(pry!(params.get()).get_info());
        let system_info = pry!(SystemInfo::read_message(request));
        println!("{:#?}", system_info);

        Promise::ok(())
    }
}

impl Drop for ServerRpcImpl {
    fn drop(&mut self) {
        self.client.disconnect();
    }
}

/// Grant access to other services
#[derive(new)]
pub struct GatewayImpl {
    client_manager: ClientManager,
}

impl gateway::Server for GatewayImpl {
    fn register_client(
        &mut self,
        params: gateway::RegisterClientParams,
        mut results: gateway::RegisterClientResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let params = pry!(pry!(params.get()).get_params());
        let client = pry!(params.get_client());
        let token = pry!(pry!(params.get_token()).to_string());

        // TODO: add authentication with JWT, identifier should be JWT within.
        let client = self.client_manager.add_client(&token, client);

        let server_access: server_rpc::Client = capnp_rpc::new_client(ServerRpcImpl::new(client));
        results.get().set_rpc(server_access);

        Promise::ok(())
    }
}

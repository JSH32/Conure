use conure_common::conure_rpc_capnp::client_rpc;

pub struct ClientRpcImpl;

impl client_rpc::Server for ClientRpcImpl {
    fn request_system_info(
        &mut self,
        _: client_rpc::RequestSystemInfoParams,
        _: client_rpc::RequestSystemInfoResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        todo!()
    }

    fn start_shell(
        &mut self,
        _: client_rpc::StartShellParams,
        _: client_rpc::StartShellResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        todo!()
    }
}

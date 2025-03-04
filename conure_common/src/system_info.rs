use crate::conure_rpc_capnp::{self, system_info::Reader};
use capnp::{
    message::{ReaderOptions, TypedBuilder, TypedReader},
    serialize::OwnedSegments,
};

/// Represents system information.
#[derive(Debug)]
pub struct SystemInfo {
    pub client_id: String,
    pub hostname: String,
    pub os_type: String,
    pub os_version: String,
    pub os_arch: String,
    pub current_time: i64,
    pub time_zone: String,
    pub user_name: String,
}

impl SystemInfo {
    /// Build system info into capnp message.
    pub fn build_message(&self) -> TypedBuilder<conure_rpc_capnp::system_info::Owned> {
        let mut builder = TypedBuilder::<conure_rpc_capnp::system_info::Owned>::new_default();
        let mut builder_root = builder.init_root();

        builder_root.set_client_id(&self.client_id);
        builder_root.set_hostname(&self.hostname);
        builder_root.set_os_type(&self.os_type);
        builder_root.set_os_version(&self.os_version);
        builder_root.set_os_arch(&self.os_arch);
        builder_root.set_current_time(self.current_time);
        builder_root.set_time_zone(&self.time_zone);
        builder_root.set_user_name(&self.user_name);

        builder
    }

    /// Read a message into a [`SystemInfo`] instance.
    pub fn read_message(value: Reader<'_>) -> Result<SystemInfo, capnp::Error> {
        Ok(SystemInfo {
            client_id: value.get_client_id()?.to_string()?,
            hostname: value.get_hostname()?.to_string()?,
            os_type: value.get_os_type()?.to_string()?,
            os_version: value.get_os_version()?.to_string()?,
            os_arch: value.get_os_arch()?.to_string()?,
            current_time: value.get_current_time(),
            time_zone: value.get_time_zone()?.to_string()?,
            user_name: value.get_user_name()?.to_string()?,
        })
    }
}

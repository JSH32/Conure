use log::info;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, RwLock},
};

use conure_common::conure_rpc_capnp::client_rpc;

/// A single online client.
pub struct ActiveClientInner {
    client: RwLock<Option<client_rpc::Client>>,
    identifier: String,
    manager: ClientManager,
}

/// Shared reference to the client.
/// This encapsulates the client itself and tries to prevent multiple dangling references.
#[derive(Clone)]
pub struct ActiveClient(Arc<ActiveClientInner>);

impl ActiveClient {
    /// Allow access to client without obtaining ownership or references.
    pub fn with_client<F, R>(&self, f: F) -> Result<R, capnp::Error>
    where
        F: FnOnce(&client_rpc::Client) -> Result<R, capnp::Error>,
    {
        let guard = self.client.read().unwrap();
        match &*guard {
            Some(client) => f(client).map_err(|e| e.into()),
            None => Err(capnp::Error::disconnected(
                "connection has been dropped and is invalid".to_string(),
            )),
        }
    }

    /// Disconnect the client and remove from [`ClientManager`].
    pub fn disconnect(&self) {
        self.manager.disconnect_client(&self.identifier);
    }
}

impl Deref for ActiveClient {
    type Target = ActiveClientInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Internal client manager state.
struct ClientManagerInner {
    clients: RwLock<HashMap<String, ActiveClient>>,
}

/// Manager for all active (online) clients.
/// This is a shared reference.
#[derive(Clone)]
pub struct ClientManager(Arc<ClientManagerInner>);

impl ClientManager {
    pub fn new() -> Self {
        Self(Arc::new(ClientManagerInner {
            clients: RwLock::new(HashMap::new()),
        }))
    }

    /// Add a client to the [`ClientManager`] and return a managed [`ActiveClient`]
    pub fn add_client(&self, identifier: &str, client: client_rpc::Client) -> ActiveClient {
        let mut clients = self.0.clients.write().unwrap();
        let identifier = identifier.to_owned();

        let client = ActiveClient(Arc::new(ActiveClientInner {
            client: RwLock::new(Option::Some(client)),
            identifier: identifier.clone(),
            manager: self.clone(),
        }));

        clients.insert(identifier.clone(), client.clone());

        info!("Client ({identifier}) has connected!");

        client
    }

    /// Remove a client by its identifier.
    pub fn disconnect_client(&self, identifier: &str) {
        if let Some(client) = self.0.clients.write().unwrap().remove(identifier) {
            // Disconnect the client, hopefully this is the only active reference when locks release
            *client.client.write().unwrap() = None;

            info!("Client ({identifier}) has disconnected!");
        }
    }
}

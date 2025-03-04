use log::info;
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast::{self, Receiver, Sender};

use conure_common::conure_rpc_capnp::client_rpc;

/// Bypassing capnp restrictions so we can pass the Client across threads.
/// We need to make sure we spawn all tasks which modify the client on the capnp's [`tokio::task::LocalSet`]
struct ThreadSafeClient(NonNull<client_rpc::Client>, PhantomData<client_rpc::Client>);

// Manually implement Send and Sync
unsafe impl Send for ThreadSafeClient {}
unsafe impl Sync for ThreadSafeClient {}

impl ThreadSafeClient {
    // Create a new ThreadSafeClient from a client_rpc::Client
    fn new(client: client_rpc::Client) -> Self {
        let boxed = Box::new(client); // Convert to a Box first to get stable memory location
        let ptr = Box::into_raw(boxed); // Convert to a raw pointer
        let non_null = unsafe { NonNull::new_unchecked(ptr) }; // Convert to NonNull

        ThreadSafeClient(non_null, PhantomData)
    }

    // Get a reference to the client
    unsafe fn get(&self) -> &client_rpc::Client {
        unsafe { self.0.as_ref() }
    }
}

impl Drop for ThreadSafeClient {
    fn drop(&mut self) {
        // Convert back to Box and drop it
        unsafe {
            let _ = Box::from_raw(self.0.as_ptr());
        }
    }
}

/// A single online client.
pub struct ActiveClientInner {
    client: RwLock<Option<ThreadSafeClient>>,
    identifier: String,
    manager: ClientManager,
}

/// Shared reference to the client.
/// This encapsulates the client itself and tries to prevent multiple dangling references.
#[derive(Clone)]
pub struct ActiveClient(Arc<ActiveClientInner>);

/// This is just client ID for now.
impl std::fmt::Debug for ActiveClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ActiveClient")
            .field(&self.0.identifier)
            .finish()
    }
}

impl ActiveClient {
    /// Allow access to client without obtaining ownership or references.
    pub fn with_client<F, R>(&self, f: F) -> Result<R, capnp::Error>
    where
        F: FnOnce(&client_rpc::Client) -> Result<R, capnp::Error>,
    {
        let guard = self.client.read().unwrap();
        match &*guard {
            Some(client) => {
                let client_ref = unsafe { client.get() };
                f(client_ref).map_err(|e| e.into())
            }
            None => Err(capnp::Error::disconnected(
                "connection has been dropped and is invalid".to_string(),
            )),
        }
    }

    /// Get the client's identifier.
    pub fn identifier(&self) -> String {
        self.identifier.clone()
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

/// Container mapping all IDs to their clients.
pub type ClientMap = HashMap<String, ActiveClient>;

/// Internal client manager state.
struct ClientManagerInner {
    clients: RwLock<ClientMap>,
}

/// Manager for all active (online) clients.
/// This is a shared reference.
#[derive(Clone)]
pub struct ClientManager {
    inner: Arc<ClientManagerInner>,
    updates_tx: Sender<ClientMap>,
    // Stored to keep at least one receiver alive.
    updates_rx: Arc<Receiver<ClientMap>>,
}

impl ClientManager {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(1024);
        Self {
            inner: Arc::new(ClientManagerInner {
                clients: RwLock::new(HashMap::new()),
            }),
            updates_tx: tx,
            updates_rx: Arc::new(rx),
        }
    }

    /// Get a clone of the clients map at this point in time.
    pub fn clients(&self) -> HashMap<String, ActiveClient> {
        let clients = self.inner.clients.read().unwrap();
        return clients.clone();
    }

    /// Get an async listener to listen for changed client events.
    pub fn get_listener(&self) -> Receiver<ClientMap> {
        self.updates_tx.subscribe()
    }

    /// Add a client to the [`ClientManager`] and return a managed [`ActiveClient`]
    pub fn add_client(&self, identifier: &str, client: client_rpc::Client) -> ActiveClient {
        // Wrapped so guard is dropped post mut operations.
        let (client, clients) = {
            let mut clients = self.inner.clients.write().unwrap();
            let identifier = identifier.to_owned();

            let ts_client = ThreadSafeClient::new(client);

            let client = ActiveClient(Arc::new(ActiveClientInner {
                client: RwLock::new(Option::Some(ts_client)),
                identifier: identifier.clone(),
                manager: self.clone(),
            }));

            clients.insert(identifier.clone(), client.clone());

            (client, clients.clone())
        };

        let updates_tx = self.updates_tx.clone();
        updates_tx.send(clients).unwrap();

        info!("Client ({identifier}) has connected!");

        client
    }

    /// Remove a client by its identifier.
    pub fn disconnect_client(&self, identifier: &str) {
        let clients = {
            let mut clients = self.inner.clients.write().unwrap();

            if let Some(client) = clients.remove(identifier) {
                // Disconnect the client, hopefully this is the only active reference when locks release
                *client.client.write().unwrap() = None;

                info!("Client ({identifier}) has disconnected!");
            }

            clients.clone()
        };

        let updates_tx = self.updates_tx.clone();
        updates_tx.send(clients).unwrap();
    }
}

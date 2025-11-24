use crate::{
    floe,
    query::{QueryEngine, observer::ObserverID},
    tree::{DeviceTree, TreeIDError, mutation::TreeMutationError, persist::TreePersistError},
};
use igloo_interface::{
    id::FloeRef,
    ipc::IglooMessage,
    query::{Query, QueryResult, check::QueryError},
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use std::{collections::HashSet, error::Error, thread::JoinHandle};

// we allow the large size difference since
// eval and apply are the most common operations
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum IglooRequest {
    Shutdown,

    /// Register with Igloo
    Register(kanal::Sender<IglooResponse>),

    #[allow(dead_code)]
    Unregister {
        client_id: usize,
    },

    // TODO unregister observer
    /// Evaluate a query
    Eval {
        client_id: usize,
        query_id: usize,
        query: Query,
    },

    /// Apply commands from a Floe
    FloeMessage {
        floe: FloeRef,
        msg: IglooMessage,
    },
}

#[derive(Debug, Clone)]
pub enum IglooResponse {
    /// Sent after registration
    /// Use this `client_id` for all future requests
    Registered { client_id: usize },

    #[allow(dead_code)]
    Result {
        query_id: usize,
        result: Result<QueryResult, QueryError>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum IglooError {
    #[error("Client's channel was full during registration. It will not be registered.")]
    ClientChannelFullRegistration,
    #[error("Client's channel was closed during registration. It will not be registered.")]
    ClientChannelClosedRegistration,
    #[error("Client {0}'s response channel is full")]
    ClientChannelFull(usize),
    #[error(
        "Client {0}'s response channel unexpectedly closed. Client is automatically unregistered."
    )]
    ClientChannelClosed(usize),
    #[error("Recieved request with invalid client ID: {0}")]
    InvalidClient(usize),
    #[error("Device Tree mutation error: {0}")]
    DeviceTreeMutation(#[from] TreeMutationError),
    #[error("Device Tree ID error: {0}")]
    DeviceTreeID(#[from] TreeIDError),
    #[error("Device Tree persist error: {0}")]
    DeviceTreePersist(#[from] TreePersistError),
    #[error("IO error: {0}")]
    IO(#[from] tokio::io::Error),
}

struct IglooCore {
    tree: DeviceTree,
    engine: QueryEngine,
    rx: kanal::Receiver<IglooRequest>,
    cm: ClientManager,
}

pub struct ClientManager {
    clients: Vec<Option<Client>>,
}

#[derive(Debug, Clone)]
pub struct Client {
    channel: kanal::Sender<IglooResponse>,
    observers: FxHashSet<ObserverID>,
}

pub async fn spawn() -> Result<(JoinHandle<()>, kanal::Sender<IglooRequest>), Box<dyn Error>> {
    let mut tree = DeviceTree::load()?;
    let mut engine = QueryEngine::default();
    let (tx, rx) = kanal::bounded(100);

    floe::spawn_all(&mut tree, &mut engine, &tx).await?;

    let core = IglooCore {
        tree,
        engine,
        rx,
        cm: ClientManager {
            clients: vec![None; 20],
        },
    };

    let handle = std::thread::spawn(move || {
        core.run();
    });

    Ok((handle, tx))
}

impl IglooCore {
    fn run(mut self) {
        while let Ok(req) = self.rx.recv() {
            if let IglooRequest::Shutdown = req {
                println!("CORE: Shutting down");
                break;
            }

            if let Err(e) = self.handle_request(req) {
                eprintln!("CORE: Error handling request: {e}");
            }
        }
    }

    fn handle_request(&mut self, req: IglooRequest) -> Result<(), IglooError> {
        use IglooRequest::*;
        match req {
            Shutdown => unreachable!(),
            FloeMessage { floe: from, msg } => {
                floe::handle_msg(&mut self.tree, &mut self.engine, from, msg)
            }
            Register(channel) => self.cm.register(channel),
            Unregister { client_id } => {
                let Some(client) = self.cm.unregister(client_id) else {
                    return Ok(());
                };

                self.engine.unregister(client.observers);
                Ok(())
            }
            Eval {
                client_id,
                query_id,
                query,
            } => self
                .engine
                .eval(&mut self.tree, &mut self.cm, client_id, query_id, query),
        }
    }
}

impl ClientManager {
    fn register(&mut self, channel: kanal::Sender<IglooResponse>) -> Result<(), IglooError> {
        let client_id = if let Some(free_slot) = self.clients.iter_mut().position(|o| o.is_none()) {
            free_slot
        } else {
            self.clients.push(None);
            self.clients.len() - 1
        };

        match channel.try_send(IglooResponse::Registered { client_id }) {
            Ok(true) => {
                self.clients[client_id] = Some(Client {
                    channel,
                    observers: HashSet::with_capacity_and_hasher(10, FxBuildHasher),
                });
                Ok(())
            }
            Ok(false) => Err(IglooError::ClientChannelFullRegistration),
            Err(_) => Err(IglooError::ClientChannelClosedRegistration),
        }
    }

    fn unregister(&mut self, client_id: usize) -> Option<Client> {
        self.clients.get_mut(client_id).and_then(|o| o.take())
    }

    pub fn send(&mut self, client_id: usize, response: IglooResponse) -> Result<(), IglooError> {
        let Some(Some(client)) = self.clients.get(client_id) else {
            return Err(IglooError::InvalidClient(client_id));
        };

        match client.channel.try_send(response) {
            Ok(true) => Ok(()),
            // TODO if client channel is full for long enough, drop the client
            Ok(false) => Err(IglooError::ClientChannelFull(client_id)),
            Err(_) => {
                self.unregister(client_id);
                Err(IglooError::ClientChannelClosed(client_id))
            }
        }
    }

    pub fn add_observer(&mut self, client_id: usize, observer_id: usize) -> Result<(), IglooError> {
        match self.clients.get_mut(client_id) {
            Some(Some(client)) => {
                client.observers.insert(observer_id);
                Ok(())
            }
            _ => Err(IglooError::InvalidClient(client_id)),
        }
    }
}

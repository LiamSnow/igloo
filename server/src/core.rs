use crate::{
    ext,
    query::{QueryEngine, watch::WatcherID},
    tree::{DeviceTree, TreeIDError, mutation::TreeMutationError, persist::TreePersistError},
};
use igloo_interface::{
    id::ExtensionIndex,
    ipc::IglooMessage,
    query::{OneShotQuery, QueryResult, WatchQuery, WatchUpdate, check::QueryError},
};
use std::{error::Error, mem, thread::JoinHandle};

// we allow the large size difference since
// eval and apply are the most common operations
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum IglooRequest {
    Shutdown,

    /// Register with Igloo
    RegisterClient(kanal::Sender<IglooResponse>),

    UnregisterClient {
        client_id: usize,
    },

    EvalOneShot {
        client_id: usize,
        query_id: usize,
        query: OneShotQuery,
    },

    SubWatch {
        client_id: usize,
        query_id: usize,
        query: WatchQuery,
    },

    UnsubWatches {
        client_id: usize,
    },

    HandleMessage {
        sender: ExtensionIndex,
        content: IglooMessage,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum IglooResponse {
    /// Sent after registration
    /// Use this `client_id` for all future requests
    Registered {
        client_id: usize,
    },

    QueryResult {
        query_id: usize,
        result: Result<QueryResult, QueryError>,
    },

    WatchUpdate {
        query_id: usize,
        value: WatchUpdate,
    },

    InvalidWatch {
        query_id: usize,
        error: QueryError,
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

// TODO client manager needs to use generational arena
pub struct ClientManager {
    clients: Vec<Option<Client>>,
}

#[derive(Debug, Clone)]
pub struct Client {
    channel: kanal::Sender<IglooResponse>,
    watchers: Vec<WatcherID>,
}

pub async fn spawn() -> Result<(JoinHandle<()>, kanal::Sender<IglooRequest>), Box<dyn Error>> {
    let mut tree = DeviceTree::load()?;
    let mut engine = QueryEngine::default();
    let (tx, rx) = kanal::bounded(100);
    let mut cm = ClientManager {
        clients: vec![None; 20],
    };

    ext::spawn_all(&mut cm, &mut tree, &mut engine, &tx).await?;

    let core = IglooCore {
        tree,
        engine,
        rx,
        cm,
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
                // for device in self.tree.devices().iter() {
                //     println!("> device: {:?}", device.id());
                //     for entity in device.entities() {
                //         println!("   - entity: {}", entity.id());
                //         for comp in entity.components() {
                //             println!("      - comp: {comp:?}");
                //         }
                //     }
                // }
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

            // extension proxy
            HandleMessage {
                sender: from,
                content: msg,
            } => ext::handle_msg(&mut self.cm, &mut self.tree, &mut self.engine, from, msg),

            // client reg
            RegisterClient(channel) => self.cm.register(channel),
            UnregisterClient { client_id } => {
                let client = self.cm.unregister(client_id)?;
                self.engine.unsub_watches(client_id, client.watchers)
            }

            // oneshot
            EvalOneShot {
                client_id,
                query_id,
                query,
            } => self
                .engine
                .eval_oneshot(&mut self.tree, &mut self.cm, client_id, query_id, query),

            // watch
            SubWatch {
                client_id,
                query_id,
                query,
            } => self
                .engine
                .sub_watch(&mut self.tree, &mut self.cm, client_id, query_id, query),
            UnsubWatches { client_id } => {
                let client = self.cm.get_client_mut(client_id)?;
                self.engine
                    .unsub_watches(client_id, mem::take(&mut client.watchers))
            }
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
                    watchers: Vec::with_capacity(5),
                });
                Ok(())
            }
            Ok(false) => Err(IglooError::ClientChannelFullRegistration),
            Err(_) => Err(IglooError::ClientChannelClosedRegistration),
        }
    }

    fn unregister(&mut self, client_id: usize) -> Result<Client, IglooError> {
        match self.clients.get_mut(client_id).and_then(|o| o.take()) {
            Some(client) => Ok(client),
            None => Err(IglooError::InvalidClient(client_id)),
        }
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
                let _ = self.unregister(client_id);
                Err(IglooError::ClientChannelClosed(client_id))
            }
        }
    }

    fn get_client_mut(&mut self, client_id: usize) -> Result<&mut Client, IglooError> {
        match self.clients.get_mut(client_id) {
            Some(Some(client)) => Ok(client),
            _ => Err(IglooError::InvalidClient(client_id)),
        }
    }

    pub fn add_watcher(
        &mut self,
        client_id: usize,
        watcher_id: WatcherID,
    ) -> Result<(), IglooError> {
        let client = self.get_client_mut(client_id)?;
        client.watchers.push(watcher_id);
        Ok(())
    }
}

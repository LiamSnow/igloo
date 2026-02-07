use crate::{
    ext::{self, ExtensionRequest},
    query::{QueryEngine, watch::WatcherID},
    tree::{DeviceTree, TreeIDError, mutation::TreeMutationError, persist::TreePersistError},
};
use igloo_interface::{
    id::{DeviceID, EntityID, EntityIndex, ExtensionIndex, GroupID},
    ipc::{ExtensionToIgloo, IglooToExtension},
    query::{OneShotQuery, QueryResult, WatchQuery, WatchUpdate, check::QueryError},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, mem, thread::JoinHandle};

/// (Client | Ext) -> Igloo Core
#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum IglooRequest {
    Shutdown,

    RegisterClient(kanal::Sender<IglooResponse>),

    Client {
        client_id: usize,
        msg: ClientMsg,
    },

    Ext {
        sender: ExtensionIndex,
        content: ExtensionToIgloo,
    },
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Unregister,

    /// evaluate one-shot query
    Eval {
        query_id: usize,
        query: OneShotQuery,
    },

    /// subscribe to watch query
    Sub {
        query_id: usize,
        query: WatchQuery,
    },

    // unsubscribe from all watch queries
    UnsubAll,

    RenameDevice {
        device_id: DeviceID,
        new_name: String,
    },

    CreateGroup {
        name: String,
    },

    DeleteGroup(GroupID),

    RenameGroup {
        group_id: GroupID,
        new_name: String,
    },

    AddDeviceToGroup(GroupID, DeviceID),

    RemoveDeviceFromGroup(GroupID, DeviceID),

    DetachExt(ExtensionIndex),
}

/// Igloo Core -> Client
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IglooResponse {
    /// Sent after registration
    /// Use this `client_id` for all future requests
    Registered {
        client_id: usize,
    },

    // one-shot queries
    EvalResult {
        query_id: usize,
        result: Result<QueryResult, QueryError>,
    },

    // watch queries
    WatchUpdate {
        query_id: usize,
        value: WatchUpdate,
    },
    WatchError {
        query_id: usize,
        error: QueryError,
    },

    // device tree mutation proxy
    InvalidID(TreeIDError),
    GroupCreated(GroupID),
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
                for device in self.tree.devices().iter() {
                    println!("> device: {:?}", device.id());
                    for entity in device.entities() {
                        println!("   - entity: {}", entity.id());
                        for comp in entity.components() {
                            println!("      - comp: {comp:?}");
                        }
                    }
                }
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
            Ext {
                sender: from,
                content: msg,
            } => self.handle_ext_msg(from, msg),

            // client reg
            RegisterClient(channel) => self.cm.register(channel),

            Client { client_id, msg } => {
                let res = self.handle_client_msg(client_id, msg);
                // all igloo errors an internal issues (ex. saving)
                // except tree ID errors (client used invalid ID)
                if let Err(IglooError::DeviceTreeID(e)) = res {
                    return self.cm.send(client_id, IglooResponse::InvalidID(e));
                }
                res
            }
        }
    }

    pub fn handle_ext_msg(
        &mut self,
        xindex: ExtensionIndex,
        msg: ExtensionToIgloo,
    ) -> Result<(), IglooError> {
        use ExtensionToIgloo::*;
        match msg {
            CreateDevice { name } => {
                let id = self.tree.create_device(
                    &mut self.cm,
                    &mut self.engine,
                    name.clone(),
                    xindex,
                )?;
                let ext = self.tree.ext(&xindex)?;
                let msg = ExtensionRequest::Msg(IglooToExtension::DeviceCreated {
                    name,
                    id: *id.inner(),
                });

                if ext.channel.try_send(msg).is_err() {
                    self.tree
                        .detach_ext(&mut self.cm, &mut self.engine, xindex, true)?;
                }
                Ok(())
            }

            RegisterEntity {
                device,
                entity_id,
                entity_index,
            } => self.tree.register_entity(
                &mut self.cm,
                &mut self.engine,
                DeviceID::new(device),
                EntityID(entity_id),
                EntityIndex(entity_index),
            ),

            WriteComponents {
                device,
                entity,
                comps,
            } => self.tree.write_components(
                &mut self.cm,
                &mut self.engine,
                DeviceID::new(device),
                EntityIndex(entity),
                comps,
            ),

            WhatsUpIgloo { .. } => {
                // TODO return err
                Ok(())
            }
        }
    }

    fn handle_client_msg(&mut self, client_id: usize, msg: ClientMsg) -> Result<(), IglooError> {
        use ClientMsg::*;
        match msg {
            Unregister => {
                let client = self.cm.unregister(client_id)?;
                self.engine.unsub_watches(client_id, client.watchers)
            }

            // one-shot queries
            Eval { query_id, query } => {
                self.engine
                    .eval_oneshot(&mut self.tree, &mut self.cm, client_id, query_id, query)
            }

            // watch queries
            Sub { query_id, query } => {
                self.engine
                    .sub_watch(&mut self.tree, &mut self.cm, client_id, query_id, query)
            }
            UnsubAll => {
                let client = self.cm.get_client_mut(client_id)?;
                self.engine
                    .unsub_watches(client_id, mem::take(&mut client.watchers))
            }

            // device tree mutation proxy
            RenameDevice {
                device_id,
                new_name,
            } => self
                .tree
                .rename_device(&mut self.cm, &mut self.engine, device_id, new_name),
            CreateGroup { name } => {
                let gid = self
                    .tree
                    .create_group(&mut self.cm, &mut self.engine, name)?;
                self.cm.send(client_id, IglooResponse::GroupCreated(gid))?;
                Ok(())
            }
            DeleteGroup(gid) => self.tree.delete_group(&mut self.cm, &mut self.engine, gid),
            RenameGroup { group_id, new_name } => {
                self.tree
                    .rename_group(&mut self.cm, &mut self.engine, &group_id, new_name)
            }
            AddDeviceToGroup(gid, did) => {
                self.tree
                    .add_device_to_group(&mut self.cm, &mut self.engine, gid, did)
            }
            RemoveDeviceFromGroup(gid, did) => {
                self.tree
                    .remove_device_from_group(&mut self.cm, &mut self.engine, gid, did)
            }
            DetachExt(xindex) => {
                self.tree
                    .detach_ext(&mut self.cm, &mut self.engine, xindex, false)
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

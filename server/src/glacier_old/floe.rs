use std::{path::Path, process::Stdio};

use futures_util::StreamExt;
use igloo_interface::{
    ComponentType, DESELECT_ENTITY, END_TRANSACTION, FloeCodec, FloeReaderDefault, FloeWriter,
    FloeWriterDefault, MAX_SUPPORTED_COMPONENT, REGISTER_ENTITY, RegisterEntity, SELECT_ENTITY,
    START_DEVICE_TRANSACTION, START_REGISTRATION_TRANSACTION, SelectEntity, StartDeviceTransaction,
    StartRegistrationTransaction, WHATS_UP_IGLOO, WRITE_INT, WhatsUpIgloo, read_component,
};
use smallvec::SmallVec;
use tokio::{
    fs,
    io::BufWriter,
    net::UnixListener,
    process::{Child, Command},
    sync::mpsc,
};
use tokio_util::codec::FramedRead;

use crate::glacier::{
    DeviceInfo,
    entity::{Entity, HasComponent},
    query::{LocalArea, LocalQuery, QueryKind},
};

struct FloeManager {
    name: String,
    idx: usize,
    reg_dev_tx: mpsc::Sender<(String, DeviceInfo)>,
    query_rx: mpsc::Receiver<LocalQuery>,
    writer: FloeWriterDefault,
    reader: FloeReaderDefault,
    devices: Vec<Device>,
}

#[derive(Debug, Default)]
pub struct Device {
    entities: Entities,
    presense: Presense,
}

pub type Entities = SmallVec<[Entity; 16]>;

#[derive(Debug, Default)]
pub struct Presense([u32; MAX_SUPPORTED_COMPONENT.div_ceil(32) as usize]);

impl Presense {
    #[inline]
    fn set(&mut self, typ: ComponentType) {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        self.0[index] |= 1u32 << bit;
    }
}

impl HasComponent for Presense {
    #[inline]
    fn has(&self, typ: ComponentType) -> bool {
        let type_id = typ as usize;
        let index = type_id >> 5;
        let bit = type_id & 31;
        (self.0[index] & (1u32 << bit)) != 0
    }
}

// TODO remove unwraps and panics
pub async fn spawn(
    name: String,
    idx: usize,
    reg_dev_tx: mpsc::Sender<(String, DeviceInfo)>,
    query_rx: mpsc::Receiver<LocalQuery>,
) -> Result<(), std::io::Error> {
    println!("Spawning Floe '{name}'");

    let cwd = format!("./floes/{name}");

    let data_path = format!("{cwd}/data");
    fs::create_dir_all(&data_path).await?;

    let socket_path = format!("{cwd}/floe.sock");
    let _ = fs::remove_file(&socket_path).await;
    let listener = UnixListener::bind(&socket_path)?;

    let mut process = Command::new(Path::new("./floe"))
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    proxy_stdio(&mut process, name.to_string());

    let (stream, _) = listener.accept().await?;
    let (reader, writer) = stream.into_split();
    let writer = FloeWriter(BufWriter::new(writer));
    let reader = FramedRead::new(reader, FloeCodec::new());

    tokio::spawn(async move {
        let man = FloeManager {
            name,
            idx,
            reg_dev_tx,
            query_rx,
            writer,
            reader,
            devices: Vec::with_capacity(32),
        };
        man.run().await;
    });

    Ok(())
}

impl FloeManager {
    async fn run(mut self) {
        // TODO implement max_supported_componets
        let max_supported_component = match self.reader.next().await {
            Some(Ok((WHATS_UP_IGLOO, payload))) => {
                let res: WhatsUpIgloo = borsh::from_slice(&payload).unwrap();

                if res.max_supported_component > MAX_SUPPORTED_COMPONENT {
                    panic!(
                        "Floe '{}' has a newer protocol than Igloo. Please upgrade Igloo",
                        self.name
                    )
                }

                println!("Floe '{}' initialized!!!", self.name);
                res.max_supported_component
            }
            Some(Ok((cmd_id, _))) => {
                panic!("Floe '{}' didn't init. Sent {cmd_id} instead.", self.name)
            }
            Some(Err(e)) => {
                panic!("Failed to read Floe '{}'s init message: {e}", self.name)
            }
            None => {
                panic!("Floe '{}' immediately closed the socket!", self.name)
            }
        };

        loop {
            tokio::select! {
                Some(res) = self.reader.next() => {
                    let (cmd_id, payload) = match res {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Error reading frame from Floe '{}': {e}", self.name);
                            continue;
                        }
                    };

                    match cmd_id {
                        START_REGISTRATION_TRANSACTION => {
                            let params: StartRegistrationTransaction = borsh::from_slice(&payload).unwrap();
                            self.handle_registration_transaction(params).await;
                        }

                        START_DEVICE_TRANSACTION => {
                            let params: StartDeviceTransaction = borsh::from_slice(&payload).unwrap();
                            self.handle_device_transaction(params).await;
                        }

                        cmd_id => {
                            eprintln!("Unexpected command {cmd_id} from Floe '{}'", self.name);
                        }
                    }
                },

                Some(req) = self.query_rx.recv() => {
                    self.handle_query_request(req).await;
                }
            }
        }
    }

    async fn handle_query_request(&mut self, req: LocalQuery) {
        match req.area {
            LocalArea::All => {
                for device_idx in 0..self.devices.len() {
                    self.handle_query_request_dev(&req, device_idx as u16, None)
                        .await;
                }
            }
            LocalArea::Device(device_idx) => {
                self.handle_query_request_dev(&req, device_idx, None).await;
            }
            LocalArea::Entity(device_idx, entity_idx) => {
                self.handle_query_request_dev(&req, device_idx, Some(entity_idx))
                    .await;
            }
        }
    }

    async fn handle_query_request_dev(
        &mut self,
        req: &LocalQuery,
        device_idx: u16,
        entity_idx: Option<u16>,
    ) {
        let device = &mut self.devices[device_idx as usize];

        // quick precheck
        if !device.presense.matches_filter(&req.filter) {
            return;
        }

        println!(
            "Query dispatch to start transaction latency: {:?}",
            req.started_at.elapsed()
        );

        self.writer
            .start_device_transaction(&StartDeviceTransaction { device_idx })
            .await
            .unwrap();

        match entity_idx {
            Some(entity_idx) => {
                Self::handle_query_request_entity(&mut self.writer, req, device, entity_idx).await;
            }
            None => {
                let len = device.entities.len() as u16;
                for entity_idx in 0..len {
                    Self::handle_query_request_entity(&mut self.writer, req, device, entity_idx)
                        .await;
                }
            }
        }

        self.writer.end_transaction().await.unwrap();
        self.writer.flush().await.unwrap();
        println!(
            "Query dispatch to end transaction latency: {:?}",
            req.started_at.elapsed()
        );
    }

    async fn handle_query_request_entity(
        writer: &mut FloeWriterDefault,
        req: &LocalQuery,
        device: &mut Device,
        entity_idx: u16,
    ) {
        let entity = &mut device.entities[entity_idx as usize];

        if !entity.matches_filter(&req.filter) {
            return;
        }

        match &req.kind {
            QueryKind::Set(comps) => {
                writer
                    .select_entity(&SelectEntity { entity_idx })
                    .await
                    .unwrap();

                for comp in comps {
                    writer.write_component(comp).await.unwrap();
                }

                writer.deselect_entity().await.unwrap();
            }
        }
    }

    async fn handle_registration_transaction(&mut self, params: StartRegistrationTransaction) {
        if params.device_idx as usize != self.devices.len() {
            panic!(
                "Floe '{}' malformed. Tried to register new device under idx={} but should have been {}",
                self.name,
                params.device_idx,
                self.devices.len()
            );
        }

        let mut entities = Entities::default();
        let mut presense = Presense::default();
        let mut entity_names = Vec::new();
        let mut selected_entity: Option<&mut Entity> = None;

        while let Some(res) = self.reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error reading frame from Floe '{}': {e}", self.name);
                    continue;
                }
            };

            if cmd_id > WRITE_INT {
                match &mut selected_entity {
                    Some(entity) => {
                        let val = read_component(cmd_id, payload).unwrap();
                        // set the entity, if we added
                        // something new, register it in presense
                        if let Some(comp_typ) = entity.set(val) {
                            presense.set(comp_typ);
                        }
                        continue;
                    }
                    None => {
                        panic!(
                            "Floe '{}' malformed when registering device '{}'. Tried to write component without an entity selected.",
                            self.name, params.initial_name
                        );
                    }
                }
            }

            match cmd_id {
                REGISTER_ENTITY => {
                    let rep: RegisterEntity = borsh::from_slice(&payload).unwrap();

                    if rep.entity_idx as usize != entities.len() {
                        panic!(
                            "Floe '{}' malformed when registering device '{}'. Tried to register entity under idx={} but should have been {}",
                            self.name,
                            params.initial_name,
                            rep.entity_idx,
                            entities.len(),
                        );
                    }

                    entity_names.push(rep.entity_name);
                    selected_entity = None;
                    entities.push(Entity::new(rep.entity_name));
                    // TODO should this also select the entity?
                }

                SELECT_ENTITY => {
                    let sep: SelectEntity = borsh::from_slice(&payload).unwrap();
                    let entity_idx = sep.entity_idx as usize;
                    if entity_idx > entities.len() - 1 {
                        panic!(
                            "Floe '{}' malformed when registering device '{}'. Tried to select entity idx={} which is not registered.",
                            self.name, params.initial_name, sep.entity_idx,
                        );
                    }
                    selected_entity = Some(entities.get_mut(entity_idx).unwrap());
                }

                DESELECT_ENTITY => {
                    selected_entity = None;
                }

                END_TRANSACTION => {
                    break;
                }

                cmd_id => {
                    panic!(
                        "Floe '{}' malformed when registering device '{}'. Sent unexpected command {cmd_id}",
                        self.name, params.initial_name,
                    );
                }
            }
        }

        self.devices.push(Device { entities, presense });

        let global_id = format!("{}-{}", self.name, params.device_id);
        let info = DeviceInfo {
            name: params.initial_name,
            idx: params.device_idx,
            entity_names,
            floe_idx: self.idx,
        };

        self.reg_dev_tx.send((global_id, info)).await.unwrap();
    }

    async fn handle_device_transaction(&mut self, params: StartDeviceTransaction) {
        let device_idx = params.device_idx as usize;
        if device_idx > self.devices.len() - 1 {
            panic!(
                "Floe '{}' malformed. Tried to start device transaction with invalid device idx={}",
                self.name, params.device_idx
            );
        }

        let device = self.devices.get_mut(device_idx).unwrap();
        let mut selected_entity: Option<&mut Entity> = None;

        while let Some(res) = self.reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error reading frame from Floe '{}': {e}", self.name);
                    continue;
                }
            };

            if cmd_id > WRITE_INT {
                match &mut selected_entity {
                    Some(entity) => {
                        let val = read_component(cmd_id, payload).unwrap();
                        // set the entity, if we added
                        // something new, register it in presense
                        if let Some(comp_typ) = entity.set(val) {
                            device.presense.set(comp_typ);
                        }
                        continue;
                    }
                    None => {
                        panic!(
                            "Floe '{}' malformed during a transaction with device idx={device_idx}. Tried to write component without an entity selected.",
                            self.name,
                        );
                    }
                }
            }

            match cmd_id {
                SELECT_ENTITY => {
                    let params: SelectEntity = borsh::from_slice(&payload).unwrap();
                    let entity_idx = params.entity_idx as usize;
                    if entity_idx > device.entities.len() - 1 {
                        panic!(
                            "Floe '{}' malformed during a transaction with device idx={device_idx}. Tried to select entity idx={entity_idx} which is not registered.",
                            self.name
                        );
                    }
                    selected_entity = Some(device.entities.get_mut(entity_idx).unwrap());
                }

                DESELECT_ENTITY => {
                    selected_entity = None;
                }

                END_TRANSACTION => {
                    break;
                }

                cmd_id => {
                    panic!(
                        "Floe '{}' malformed during a transaction with device idx={device_idx}. Sent unexpected command {cmd_id}",
                        self.name,
                    );
                }
            }
        }
    }
}

/// Proxies stdout and stderr to this process prefixed with Floe's name
fn proxy_stdio(child: &mut Child, name: String) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdout) = stdout {
        let name_stdout = name.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[{name_stdout}] {line}");
            }
        });
    }

    if let Some(stderr) = stderr {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[{name}] {line}");
            }
        });
    }
}

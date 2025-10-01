use std::collections::HashMap;

use igloo_interface::{
    CustomCommandErrorPayload, DeviceRegisteredPayload, ExecuteCustomCommandPayload,
    HandshakeRequestPayload, RegisterDevicePayload, RequestUpdatesPayload,
    floe::{FloeHandler, FloeManager},
};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::mpsc};
use uuid::Uuid;

use crate::{
    CONFIG_FILE,
    connection::base::Connectionable,
    device::{ConnectionParams, Device, EntityData},
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Config {
    /// maps Igloo ID -> Connection Params
    device_map: HashMap<Uuid, ConnectionParams>,
}

#[derive(Default)]
pub struct ESPHomeFloe {
    pub config: Config,
    pub devices_tx: HashMap<u16, mpsc::Sender<RequestUpdatesPayload>>,
    /// devices which have been requested to be registered,
    /// but havn't been registered yet
    pub pending_devices: HashMap<String, (Device, Vec<EntityData>)>,
}

impl FloeHandler for ESPHomeFloe {
    async fn on_handshake(&mut self, _payload: HandshakeRequestPayload, manager: &FloeManager) {
        // igloos ready -> request to register all our known devices
        for (device_id, params) in self.config.device_map.clone() {
            self.register_device(params, device_id, manager).await;
        }
    }

    async fn on_device_registered(
        &mut self,
        payload: DeviceRegisteredPayload,
        manager: &FloeManager,
    ) {
        let Some((mut device, entities)) = self.pending_devices.remove(&payload.id) else {
            manager
                .log(format!("Igloo said I dev {} but I didn't", payload.id))
                .await
                .unwrap();
            return;
        };

        device
            .init(payload.device_ref, payload.entity_refs, entities)
            .await
            .unwrap(); // FIXME unwrap

        // make channel so we can send commands
        let (cmd_tx, cmd_rx) = mpsc::channel(10);
        self.devices_tx.insert(payload.device_ref, cmd_tx);

        tokio::spawn(async move {
            device.run(cmd_rx).await.unwrap(); // FIXME unwrap

            // TODO log when device dies, maybe unregister?
        });
    }

    async fn on_request_updates(&mut self, payload: RequestUpdatesPayload, _: &FloeManager) {
        let chan = self.devices_tx.get(&payload.device).unwrap(); // FIXME
        chan.try_send(payload).unwrap();
    }

    async fn on_execute_custom_command(
        &mut self,
        payload: ExecuteCustomCommandPayload,
        manager: &FloeManager,
    ) {
        // TODO make enum?

        match payload.command_id {
            // TODO we should probably add_noise_device and add_plain_device
            // then just take simple two args comma-separated
            0 => {
                self.handle_add_device(payload.data, manager).await;
            }
            cmd_id => {
                manager
                    .custom_command_error(CustomCommandErrorPayload {
                        error_code: 0,
                        message: format!("Unknown custom command: {cmd_id}"),
                    })
                    .await
                    .unwrap();
            }
        }
    }
}

impl ESPHomeFloe {
    async fn register_device(
        &mut self,
        params: ConnectionParams,
        device_id: Uuid,
        manager: &FloeManager,
    ) {
        manager
            .log(format!("Connecting to device with params = {params:#?}"))
            .await
            .unwrap();

        let mut device = Device::new(params.clone(), manager.clone());
        let entities = device.connect().await.unwrap(); // FIXME unwrap
        let device_name = device
            .connection
            .get_name()
            .unwrap_or("unnamed_esphome_device".to_string());

        let entity_names: Vec<String> = entities.iter().map(|e| e.name.to_string()).collect();

        manager
            .register_device(RegisterDevicePayload {
                id: device_id.to_string(),
                initial_name: device_name,
                entity_names,
            })
            .await
            .unwrap();

        self.pending_devices
            .insert(device_id.to_string(), (device, entities));
    }

    async fn handle_add_device(&mut self, payload: Vec<u8>, manager: &FloeManager) {
        let payload = String::from_utf8(payload).unwrap(); // FIXME unwrap

        let params = match serde_json::from_str::<ConnectionParams>(&payload) {
            Ok(p) => p,
            Err(e) => {
                manager
                    .custom_command_error(CustomCommandErrorPayload {
                        error_code: 0,
                        message: format!("Failed to deserialize custom params: {e}"),
                    })
                    .await
                    .unwrap();
                return;
            }
        };

        let device_id = Uuid::now_v7();

        self.register_device(params.clone(), device_id, manager)
            .await;

        self.config.device_map.insert(device_id, params);

        fs::write(CONFIG_FILE, serde_json::to_string(&self.config).unwrap())
            .await
            .unwrap(); // FIXME unwrap
    }
}

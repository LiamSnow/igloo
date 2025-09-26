use igloo_interface::{ComponentUpdate, Entities};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use uuid::Uuid;

use crate::glacier::GlacierError;

const FILE: &str = "state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DeviceTree {
    /// Zone ID -> Data
    zones: HashMap<String, Zone>,

    /// Device ID -> Data
    /// Devices here are guarenteed to exist in `.persistent_dev_data`
    #[serde(skip)]
    active_devs: HashMap<Uuid, Device>,

    /// Stores names for devices by ID persistently
    persistent_dev_data: PersistentDeviceData,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Device {
    pub floe_name: String,
    pub entities: Entities,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PersistentDeviceData {
    /// Device ID -> Name
    id_to_name: HashMap<Uuid, String>,
    /// Device Name -> ID
    name_to_id: HashMap<String, Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    /// This is displayed on the frontend BUT Queries use the Zone ID
    /// changes of Zone ID's require all references
    /// to be updates, so plz update display_name instead
    pub name: String,
    pub devices: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub enum DeviceTreeUpdate {
    AddDevice {
        floe_name: String,
        id: Uuid,
        name: String,
        entities: Entities,
    },
    ComponentUpdates {
        floe_name: String,
        updates: Vec<ComponentUpdate>,
    },
}

impl DeviceTree {
    pub async fn load() -> Result<Self, GlacierError> {
        if fs::try_exists(FILE).await? {
            let contents = fs::read_to_string(FILE).await?;
            let res = serde_json::from_str(&contents)?;
            Ok(res)
        } else {
            // TODO change to make blank
            println!("{FILE} doesn't exist, making test data.");

            let mut state = DeviceTree::default();

            state.zones.insert(
                "kitchen".to_string(),
                Zone {
                    name: "Kitchen".to_string(),
                    devices: vec![],
                },
            );

            state.save().await?;

            Ok(state)
        }
    }

    pub async fn save(&self) -> Result<(), GlacierError> {
        let contents = serde_json::to_string(self)?;
        fs::write(FILE, contents).await?;
        Ok(())
    }

    /// Adds an active device, which also adds its persistent storage if needed
    pub async fn add_device(
        &mut self,
        floe_name: String,
        id: Uuid,
        name: String,
        entities: Entities,
    ) -> Result<(), GlacierError> {
        // TODO FIXME remove
        println!(
            "{floe_name} registering {name} ({id}) with {}",
            serde_json::to_string(&entities).unwrap()
        );

        // prevent double active device registration
        if let Some(dev) = self.active_devs.get(&id) {
            eprintln!(
                "Failed to register device {id} for {floe_name}, already registered under {}!",
                dev.floe_name
            );
            // TODO what do we do now? Restart the provider? It probably thinks everythings fine
        }

        // register active device
        self.active_devs.insert(
            id,
            Device {
                floe_name,
                entities,
            },
        );

        // register persistent storage if needed
        if !self.persistent_dev_data.id_to_name.contains_key(&id) {
            self.add_persistent_dev(id, name).await?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_dev_id(&self, name: &str) -> Option<&Uuid> {
        self.persistent_dev_data.name_to_id.get(name)
    }

    #[allow(dead_code)]
    pub fn get_dev_name(&self, id: &Uuid) -> Option<&String> {
        self.persistent_dev_data.id_to_name.get(id)
    }

    #[allow(dead_code)]
    pub fn get_dev(&self, id: &Uuid) -> Option<&Device> {
        self.active_devs.get(id)
    }

    pub fn get_dev_mut(&mut self, id: &Uuid) -> Option<&mut Device> {
        self.active_devs.get_mut(id)
    }

    /// Renames a device (modifies persistent storage)
    #[allow(dead_code)]
    pub async fn rename_dev(
        &mut self,
        id: &Uuid,
        new_name: String,
        keep_dangling: bool,
    ) -> Result<(), GlacierError> {
        let Some(old_name) = self.persistent_dev_data.id_to_name.get(id) else {
            return Ok(()); // dev doesn't exist
        };

        if !keep_dangling {
            self.persistent_dev_data.name_to_id.remove(old_name);
        }

        self.add_persistent_dev(*id, new_name).await
    }

    /// Adds device to persistent storage and saves
    async fn add_persistent_dev(&mut self, id: Uuid, mut name: String) -> Result<(), GlacierError> {
        self.make_name_unqiue(&mut name);
        self.persistent_dev_data.id_to_name.insert(id, name.clone());
        self.persistent_dev_data.name_to_id.insert(name, id);
        self.save().await
    }

    /// Appends "_1" to a device name until it finds a unique name
    fn make_name_unqiue(&self, name: &mut String) {
        loop {
            if !self.persistent_dev_data.name_to_id.contains_key(name) {
                break;
            }
            name.push_str("_1");
        }
    }

    #[allow(dead_code)]
    pub fn get_zone(&self, id: &str) -> Option<&Zone> {
        self.zones.get(id)
    }

    /// Removes a zone and saves
    #[allow(dead_code)]
    pub async fn remove_zone(&mut self, id: &str) -> Result<Option<Zone>, GlacierError> {
        let Some(zone) = self.zones.remove(id) else {
            return Ok(None);
        };
        self.save().await?;
        Ok(Some(zone))
    }

    /// changes a zone ID and saves, returns `false` if zone doesn't exist
    #[allow(dead_code)]
    pub async fn change_zone_id(
        &mut self,
        old_id: &str,
        new_id: String,
    ) -> Result<bool, GlacierError> {
        let Some(zone) = self.zones.remove(old_id) else {
            return Ok(false);
        };
        self.zones.insert(new_id, zone);
        self.save().await?;
        Ok(true)
    }

    #[allow(dead_code)]
    pub async fn change_zone_name(
        &mut self,
        zone_id: &str,
        new_name: String,
    ) -> Result<(), GlacierError> {
        let Some(zone) = self.zones.get_mut(zone_id) else {
            return Ok(());
        };
        zone.name = new_name;
        self.save().await
    }

    #[allow(dead_code)]
    pub async fn remove_dev_from_zone(
        &mut self,
        device_id: &Uuid,
        zone_id: &str,
    ) -> Result<(), GlacierError> {
        // TODO error if zone doesn't exist?
        let Some(zone) = self.zones.get_mut(zone_id) else {
            return Ok(());
        };
        let Some(pos) = zone.devices.iter().position(|&d| d == *device_id) else {
            return Ok(());
        };
        zone.devices.swap_remove(pos);
        self.save().await
    }

    #[allow(dead_code)]
    pub async fn add_dev_to_zone(
        &mut self,
        device_id: Uuid,
        zone_id: &str,
    ) -> Result<(), GlacierError> {
        // TODO error if zone doesn't exist?
        let Some(zone) = self.zones.get_mut(zone_id) else {
            return Ok(());
        };
        zone.devices.push(device_id);
        self.save().await
    }
}

use super::{Device, DeviceTree, Group};
use crate::tree::{COMP_TYPE_ARR_LEN, Presense, arena::Arena};
use igloo_interface::id::{DeviceID, FloeID, GenerationalID, GroupID};
use ini::Ini;
use rustc_hash::{FxBuildHasher, FxHashSet};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    fs,
    num::ParseIntError,
    time::Instant,
};

pub const GROUPS_FILE: &str = "groups.ini";
pub const DEVICES_FILE: &str = "devices.ini";

// TODO custom parser with:
//  - Strict ordering
//  - Hella Cows
//  - miette error messages

#[derive(thiserror::Error, Debug)]
pub enum TreePersistError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("Ini parse error: {0}")]
    IniParse(#[from] ini::ParseError),

    #[error("Missing generation field at top of file")]
    MissingGlobalGeneration,
    #[error("Bad global generation field. Expected Integer. Error: {0}")]
    BadGlobalGeneration(ParseIntError),

    #[error("Bad Group Index: '{0}'. Expected Integer. Error: {1}")]
    GroupBadIndex(String, ParseIntError),
    #[error("Group #{0} missing name field")]
    GroupMissingName(usize),
    #[error("Group #{0} '{1}' missing generation field")]
    GroupMissingGeneration(usize, String),
    #[error("Group #{0} '{1}' has bad generation field. Expected Integer. Error: {2}")]
    GroupBadGeneration(usize, String, ParseIntError),
    #[error("Group #{0} '{1}''s device {2} missing index part. Expected 'idx:generation'")]
    GroupDeviceMissingIndex(usize, String, String),
    #[error("Group #{0} '{1}''s device {2} missing generation part. Expected 'idx:generation'")]
    GroupDeviceMissingGeneration(usize, String, String),
    #[error(
        "Group #{0} '{1}''s device {2} bad index part. Expected 'idx:generation', both Integers. Error: {3}"
    )]
    GroupDeviceBadIndex(usize, String, String, ParseIntError),
    #[error(
        "Group #{0} '{1}''s device {2} bad generation part. Expected 'idx:generation', both Integers. Error: {3}"
    )]
    GroupDeviceBadGeneration(usize, String, String, ParseIntError),

    #[error("Bad Device Index: '{0}'. Expected Integer. Error: {1}")]
    DeviceBadIndex(String, ParseIntError),
    #[error("Device #{0} missing name field")]
    DeviceMissingName(usize),
    #[error("Device #{0} missing generation field")]
    DeviceMissingGeneration(usize),
    #[error("Device #{0} '{1}' has bad generation field. Expected Integer. Error: {2}")]
    DeviceBadGeneration(usize, String, ParseIntError),
    #[error("Device #{0} missing owner field")]
    DeviceMissingOwner(usize),
}

impl DeviceTree {
    pub fn load() -> Result<Self, TreePersistError> {
        let groups = Self::load_groups()?;
        let devices = Self::load_devices()?;

        // put groups on devices
        let mut devices = devices;
        for group in groups.iter() {
            for did in &group.devices {
                if let Some(device) = devices.get_mut(did) {
                    device.groups.insert(group.id);
                }
            }
        }

        Ok(Self {
            groups,
            devices,
            attached_floes: Vec::with_capacity(10),
            floe_ref_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
        })
    }

    fn load_groups() -> Result<Arena<GroupID, Group>, TreePersistError> {
        if !fs::exists(GROUPS_FILE)? {
            return Ok(Arena::with_capacity(50));
        }

        let content = fs::read_to_string(GROUPS_FILE)?;
        let ini = Ini::load_from_str(&content)?;

        // read global generation
        let global_gen = ini
            .general_section()
            .get("generation")
            .ok_or(TreePersistError::MissingGlobalGeneration)?
            .parse::<u32>()
            .map_err(TreePersistError::BadGlobalGeneration)?;

        // find max index to preallocate
        let max_idx = ini
            .sections()
            .filter_map(|s| s?.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        let mut arena = Arena::with_preallocated_slots(max_idx, global_gen);

        for section in ini.sections() {
            let Some(idx_str) = section else { continue };
            let group_idx: usize = idx_str
                .parse()
                .map_err(|e| TreePersistError::GroupBadIndex(idx_str.to_string(), e))?;

            let section_data = ini.section(Some(idx_str)).unwrap();

            let group_name = section_data
                .get("name")
                .ok_or(TreePersistError::GroupMissingName(group_idx))?
                .to_string();

            let group_gen: u32 = section_data
                .get("generation")
                .ok_or(TreePersistError::GroupMissingGeneration(
                    group_idx,
                    group_name.clone(),
                ))?
                .parse()
                .map_err(|e| {
                    TreePersistError::GroupBadGeneration(group_idx, group_name.clone(), e)
                })?;

            let devices_str = section_data.get("devices").unwrap_or("");

            let devices: FxHashSet<DeviceID> = if devices_str.is_empty() {
                HashSet::with_capacity_and_hasher(20, FxBuildHasher)
            } else {
                devices_str
                    .split(',')
                    .map(|s| {
                        let mut parts = s.split(':');

                        let device_idx = parts
                            .next()
                            .ok_or_else(|| {
                                TreePersistError::GroupDeviceMissingIndex(
                                    group_idx,
                                    group_name.clone(),
                                    s.to_string(),
                                )
                            })?
                            .parse()
                            .map_err(|e| {
                                TreePersistError::GroupDeviceBadIndex(
                                    group_idx,
                                    group_name.clone(),
                                    s.to_string(),
                                    e,
                                )
                            })?;

                        let device_gen = parts
                            .next()
                            .ok_or_else(|| {
                                TreePersistError::GroupDeviceMissingGeneration(
                                    group_idx,
                                    group_name.clone(),
                                    s.to_string(),
                                )
                            })?
                            .parse()
                            .map_err(|e| {
                                TreePersistError::GroupDeviceBadGeneration(
                                    group_idx,
                                    group_name.clone(),
                                    s.to_string(),
                                    e,
                                )
                            })?;

                        Ok(DeviceID::from_parts(device_idx, device_gen))
                    })
                    .collect::<Result<_, TreePersistError>>()?
            };

            let id = GroupID::from_parts(group_idx as u32, group_gen);
            let group = Group {
                id,
                name: group_name,
                devices,
            };

            arena
                .insert_at(id, group)
                .expect("failed to insert group during load");
        }

        Ok(arena)
    }

    fn load_devices() -> Result<Arena<DeviceID, Device>, TreePersistError> {
        if !fs::exists(DEVICES_FILE)? {
            return Ok(Arena::with_capacity(200));
        }

        let content = fs::read_to_string(DEVICES_FILE)?;
        let ini = Ini::load_from_str(&content)?;

        // read global generation
        let global_gen = ini
            .general_section()
            .get("generation")
            .ok_or(TreePersistError::MissingGlobalGeneration)?
            .parse::<u32>()
            .map_err(TreePersistError::BadGlobalGeneration)?;

        // find max index to preallocate
        let max_idx = ini
            .sections()
            .filter_map(|s| s?.parse::<usize>().ok())
            .max()
            .unwrap_or(0);

        let mut arena = Arena::with_preallocated_slots(max_idx, global_gen);

        for section in ini.sections() {
            let Some(idx_str) = section else { continue };
            let device_idx: usize = idx_str
                .parse()
                .map_err(|e| TreePersistError::DeviceBadIndex(idx_str.to_string(), e))?;

            let section_data = ini.section(Some(idx_str)).unwrap();

            let device_name = section_data
                .get("name")
                .ok_or(TreePersistError::DeviceMissingName(device_idx))?
                .to_string();

            let device_gen: u32 = section_data
                .get("generation")
                .ok_or(TreePersistError::DeviceMissingGeneration(device_idx))?
                .parse()
                .map_err(|e| {
                    TreePersistError::DeviceBadGeneration(device_idx, device_name.clone(), e)
                })?;

            let owner = FloeID(
                section_data
                    .get("owner")
                    .ok_or(TreePersistError::DeviceMissingOwner(device_idx))?
                    .to_string(),
            );

            let id = DeviceID::from_parts(device_idx as u32, device_gen);
            let device = Device {
                id,
                name: device_name,
                owner,
                owner_ref: None,
                groups: FxHashSet::with_capacity_and_hasher(10, FxBuildHasher),
                presense: Presense::default(),
                entities: SmallVec::default(),
                entity_index_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
                last_updated: Instant::now(),
                comp_to_entity: [const { SmallVec::new_const() }; COMP_TYPE_ARR_LEN],
            };

            arena
                .insert_at(id, device)
                .expect("failed to insert device during load");
        }

        Ok(arena)
    }

    pub(super) fn save_groups(&self) -> Result<(), TreePersistError> {
        let mut ini = Ini::new();

        // write global generation at top
        ini.with_general_section()
            .set("generation", self.groups.generation().to_string());

        for (idx, entry) in self.groups.items().iter().enumerate() {
            let Some(group) = entry.value() else {
                continue;
            };

            ini.with_section(Some(idx.to_string()))
                .set("name", &group.name)
                .set(
                    "devices",
                    group
                        .devices
                        .iter()
                        .map(|d| format!("{}:{}", d.index(), d.generation()))
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .set("generation", group.id.generation().to_string());
        }

        let mut buf = Vec::new();
        ini.write_to(&mut buf)?;
        fs::write(GROUPS_FILE, buf)?;

        Ok(())
    }

    pub(super) fn save_devices(&self) -> Result<(), TreePersistError> {
        let mut ini = Ini::new();

        // write global generation at top
        ini.with_general_section()
            .set("generation", self.devices.generation().to_string());

        for (idx, entry) in self.devices.items().iter().enumerate() {
            let Some(device) = entry.value() else {
                continue;
            };

            ini.with_section(Some(idx.to_string()))
                .set("name", &device.name)
                .set("owner", &device.owner.0)
                .set("generation", device.id.generation().to_string());
        }

        let mut buf = Vec::new();
        ini.write_to(&mut buf)?;
        fs::write(DEVICES_FILE, buf)?;

        Ok(())
    }
}

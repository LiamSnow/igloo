use super::{Device, DeviceTree, Group};
use crate::{DATA_DIR, tree::arena::Arena};
use igloo_interface::id::{DeviceIDMarker, GroupIDMarker};
use rustc_hash::FxBuildHasher;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

pub const GROUPS_FILE: &str = "groups.toml";
pub const DEVICES_FILE: &str = "devices.toml";

#[derive(thiserror::Error, Debug)]
pub enum TreePersistError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("{} cannot be a directory", _0.to_string_lossy())]
    FileIsDirectory(PathBuf),
    #[error("`{}`: {}", _0, _1)]
    Deserialize(&'static str, toml::de::Error),
    #[error("`{}`: {}", _0, _1)]
    Serialize(&'static str, toml::ser::Error),
}

impl DeviceTree {
    pub fn load() -> Result<Self, TreePersistError> {
        let mut groups_file = Self::open_file(GROUPS_FILE)?;
        let mut devices_file = Self::open_file(DEVICES_FILE)?;

        let groups = Self::load_groups(&mut groups_file)?;
        let devices = Self::load_devices(&mut devices_file)?;

        // build device->group
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
            groups_file: Some(groups_file),
            devices,
            devices_file: Some(devices_file),
            attached_exts: Vec::with_capacity(10),
            ext_ref_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
        })
    }

    fn open_file(filename: &str) -> Result<File, TreePersistError> {
        let mut path = DATA_DIR.get().unwrap().clone();
        path.push(filename);

        if !fs::exists(&path)? {
            fs::write(&path, format!("generation = 0\n\n"))?;
        }

        let file = File::options().read(true).write(true).open(&path)?;

        let meta = file.metadata()?;
        if meta.is_dir() {
            return Err(TreePersistError::FileIsDirectory(path));
        }

        if meta.is_symlink() {
            let sym_meta = fs::symlink_metadata(&path)?;
            if sym_meta.is_dir() {
                return Err(TreePersistError::FileIsDirectory(path));
            }
        }

        Ok(file)
    }

    fn load_groups(file: &mut File) -> Result<Arena<GroupIDMarker, Group>, TreePersistError> {
        let mut content = String::with_capacity(file.metadata()?.len() as usize);
        file.read_to_string(&mut content)?;
        Ok(toml::from_str(&content).map_err(|e| TreePersistError::Deserialize(GROUPS_FILE, e))?)
    }

    fn load_devices(file: &mut File) -> Result<Arena<DeviceIDMarker, Device>, TreePersistError> {
        let mut content = String::with_capacity(file.metadata()?.len() as usize);
        file.read_to_string(&mut content)?;
        Ok(toml::from_str(&content).map_err(|e| TreePersistError::Deserialize(DEVICES_FILE, e))?)
    }

    pub(super) fn save_groups(&mut self) -> Result<(), TreePersistError> {
        if let Some(file) = &mut self.groups_file {
            write_toml(GROUPS_FILE, file, &self.groups)?;
        }
        Ok(())
    }

    pub(super) fn save_devices(&mut self) -> Result<(), TreePersistError> {
        if let Some(file) = &mut self.devices_file {
            write_toml(DEVICES_FILE, file, &self.devices)?;
        }
        Ok(())
    }
}

pub fn write_toml<S: Serialize>(
    filename: &'static str,
    file: &mut File,
    data: &S,
) -> Result<(), TreePersistError> {
    file.seek(SeekFrom::Start(0))?;
    let content =
        toml::to_string_pretty(data).map_err(|e| TreePersistError::Serialize(filename, e))?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    file.set_len(content.len() as u64)?;
    Ok(())
}

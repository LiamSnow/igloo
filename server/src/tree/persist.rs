//! This file handles saving and loading from groups.ini and devices.ini
//!
//! The goal here is to a super reliable parser with good error messages.
//! Performance doesn't really matter here since it only happens once.
//!
//! For saving our goal is pure speed since it blocks the main loop.

use super::{Device, DeviceTree, Group};
use crate::tree::arena::{Arena, SlotOccupied};
use igloo_interface::id::{DeviceID, DeviceIDMarker, ExtensionID, GroupIDMarker};
use rustc_hash::FxBuildHasher;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
};

pub const GROUPS_FILE: &str = "data/groups.toml";
pub const DEVICES_FILE: &str = "data/devices.toml";
pub const WRITER_BUF_CAPACITY: usize = 1000;

pub const HEADER_COMMENT: &str = "# WARN: Do not modify \
this file unless you really know what you're doing.\n\
# This file is NOT format or comment preserving \n\n";

#[derive(thiserror::Error, Debug)]
pub enum TreePersistError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("{0} cannot be a directory")]
    FileIsDirectory(String),
    #[error("Devices `{}` and `{}` have duplicate indicies. Hint: maybe file corruption or an error in manual modification.", _0.tried, _0.there)]
    DuplicateDevices(#[from] SlotOccupied<DeviceIDMarker>),
    #[error("Groups `{}` and `{}` have duplicate indicies. Hint: maybe file corruption or an error in manual modification.", _0.tried, _0.there)]
    DuplicateGroups(#[from] SlotOccupied<GroupIDMarker>),
}

#[derive(Deserialize)]
struct GroupsIR {
    generation: u32,
    group: Vec<Group>,
}

#[derive(Deserialize)]
struct DevicesIR {
    generation: u32,
    device: Vec<DeviceIR>,
}

#[derive(Deserialize)]
struct DeviceIR {
    id: DeviceID,
    name: String,
    owner: ExtensionID,
}

impl DeviceTree {
    pub fn load() -> Result<Self, TreePersistError> {
        let mut groups_file = Self::open_file(GROUPS_FILE)?;
        let mut devices_file = Self::open_file(DEVICES_FILE)?;

        let groups = Self::load_groups(&mut groups_file)?;
        let devices = Self::load_devices(&mut devices_file)?;

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
            groups_writer: PersistWriter::new(groups_file),
            devices,
            devices_writer: PersistWriter::new(devices_file),
            attached_exts: Vec::with_capacity(10),
            ext_ref_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
        })
    }

    fn open_file(filename: &str) -> Result<File, TreePersistError> {
        if !fs::exists(filename)? {
            fs::write(filename, "generation=0\n")?;
        }

        let file = File::options().read(true).write(true).open(filename)?;

        let meta = file.metadata()?;
        if meta.is_dir() {
            return Err(TreePersistError::FileIsDirectory(filename.to_string()));
        }

        if meta.is_symlink() {
            let sym_meta = fs::symlink_metadata(filename)?;
            if sym_meta.is_dir() {
                return Err(TreePersistError::FileIsDirectory(filename.to_string()));
            }
        }

        Ok(file)
    }

    fn load_groups(file: &mut File) -> Result<Arena<GroupIDMarker, Group>, TreePersistError> {
        let mut content = String::with_capacity(file.metadata()?.len() as usize);
        file.read_to_string(&mut content)?;
        let ir: GroupsIR = toml::from_str(&content).unwrap();

        let max_index = ir
            .group
            .last()
            .map(|last| last.id.index())
            .unwrap_or_default();
        let mut arena = Arena::new(max_index as usize, ir.generation);

        for group in ir.group {
            arena.insert_at(group.id, group)?;
        }

        Ok(arena)
    }

    fn load_devices(file: &mut File) -> Result<Arena<DeviceIDMarker, Device>, TreePersistError> {
        let mut content = String::with_capacity(file.metadata()?.len() as usize);
        file.read_to_string(&mut content)?;
        let ir: DevicesIR = toml::from_str(&content).unwrap();

        let max_index = ir
            .device
            .last()
            .map(|last| last.id.index())
            .unwrap_or_default();
        let mut arena = Arena::new(max_index as usize, ir.generation);

        for device in ir.device {
            arena.insert_at(device.id, Device::new(device.id, device.name, device.owner))?;
        }

        Ok(arena)
    }

    pub(super) fn save_groups(&mut self) -> Result<(), TreePersistError> {
        let w = &mut self.groups_writer;

        w.begin_write()?;
        w.write_header(self.groups.generation());

        for entry in self.groups.items().iter() {
            let Some(group) = entry.value() else {
                continue;
            };

            w.buf.push_str("\n[[group]]\n");
            let content = toml::to_string_pretty(group).unwrap();
            w.buf.push_str(&content);
        }

        w.end_write()?;

        Ok(())
    }

    pub(super) fn save_devices(&mut self) -> Result<(), TreePersistError> {
        let w = &mut self.devices_writer;

        w.begin_write()?;
        w.write_header(self.devices.generation());

        for entry in self.devices.items().iter() {
            let Some(device) = entry.value() else {
                continue;
            };

            w.buf.push_str("\n[[device]]\n");
            let content = toml::to_string_pretty(device).unwrap();
            w.buf.push_str(&content);
        }

        w.end_write()?;

        Ok(())
    }
}

pub struct PersistWriter {
    file: Option<File>,
    buf: String,
    itoa_buf: itoa::Buffer,
}

impl PersistWriter {
    pub fn new(file: File) -> Self {
        Self {
            file: Some(file),
            buf: String::with_capacity(WRITER_BUF_CAPACITY),
            itoa_buf: itoa::Buffer::new(),
        }
    }

    pub fn fake() -> Self {
        Self {
            file: None,
            buf: String::with_capacity(WRITER_BUF_CAPACITY),
            itoa_buf: itoa::Buffer::new(),
        }
    }

    pub fn begin_write(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.file {
            file.seek(SeekFrom::Start(0))?;
            self.buf.clear();
        }
        Ok(())
    }

    pub fn end_write(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.file {
            file.write_all(self.buf.as_bytes())?;
            file.flush()?;
            file.set_len(self.buf.len() as u64)?;
        }
        Ok(())
    }

    pub fn write_header(&mut self, generation: u32) {
        self.buf.push_str(HEADER_COMMENT);
        self.buf.push_str("generation = ");
        self.buf.push_str(self.itoa_buf.format(generation));
        self.buf.push_str("\n\n");
    }
}

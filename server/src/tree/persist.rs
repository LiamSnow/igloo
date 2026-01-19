//! This file handles saving and loading from groups.ini and devices.ini
//!
//! The goal here is to a super reliable parser with good error messages.
//! Performance doesn't really matter here since it only happens once.
//!
//! For saving our goal is pure speed since it blocks the main loop.

use super::{Device, DeviceTree, Group};
use crate::tree::arena::{Arena, InsertAtError};
use igloo_interface::id::{DeviceID, ExtensionID, GenerationalID, GroupID};
use rustc_hash::FxBuildHasher;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, Seek, SeekFrom, Write},
    num::ParseIntError,
};

pub const GROUPS_FILE: &str = "groups.ini";
pub const DEVICES_FILE: &str = "devices.ini";

pub const HEADER_COMMENT: &str = "; WARN: Do not modify \
this file unless you really know what you're doing.\n\
; This file is NOT format or comment preserving \n\n";

#[derive(thiserror::Error, Debug)]
pub enum TreePersistError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("{0} cannot be a directory")]
    FileIsDirection(String),
    #[error(transparent)]
    Parse(#[from] ContexedParseError),
}

#[derive(thiserror::Error, Debug)]
#[error("Error in {filename} on line {line}: {inner}")]
pub struct ContexedParseError {
    pub filename: String,
    pub line: usize,
    pub inner: ParseError,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    ParseInt(ParseIntError),
    #[error("Unclosed section bracket")]
    UnclosedSection,
    #[error("Invalid section number: {0}")]
    InvalidSectionNumber(String),
    #[error("Invalid line format (expected section or key=value)")]
    InvalidLineFormat,
    #[error(transparent)]
    InsertAt(InsertAtError),
    #[error("Expected {expected}, found end of file")]
    UnexpectedEOF { expected: String },
    #[error("Expected {expected}, found section [{found}]")]
    UnexpectedSection { expected: String, found: u64 },
    #[error("Expected {expected}, found field '{found_key}={found_value}'")]
    UnexpectedField {
        expected: String,
        found_key: String,
        found_value: String,
    },
}

impl DeviceTree {
    pub fn load() -> Result<Self, TreePersistError> {
        let mut groups_file = Self::open_file(GROUPS_FILE)?;
        let mut devices_file = Self::open_file(DEVICES_FILE)?;
        let groups_meta = groups_file.metadata()?;
        let devices_meta = devices_file.metadata()?;

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
            devices,
            attached_exts: Vec::with_capacity(10),
            ext_ref_lut: HashMap::with_capacity_and_hasher(10, FxBuildHasher),
            groups_writer: IniWriter::new(
                groups_file,
                usize::max(1000, (groups_meta.len() * 100 + 200) as usize),
            ),
            devices_writer: IniWriter::new(
                devices_file,
                usize::max(1000, (devices_meta.len() * 100 + 200) as usize),
            ),
        })
    }

    fn open_file(filename: &str) -> Result<File, TreePersistError> {
        if !fs::exists(filename)? {
            fs::write(filename, "generation=0\n")?;
        }

        let file = File::options().read(true).write(true).open(filename)?;

        let meta = file.metadata()?;
        if meta.is_dir() {
            return Err(TreePersistError::FileIsDirection(filename.to_string()));
        }

        if meta.is_symlink() {
            let sym_meta = fs::symlink_metadata(filename)?;
            if sym_meta.is_dir() {
                return Err(TreePersistError::FileIsDirection(filename.to_string()));
            }
        }

        Ok(file)
    }

    fn load_groups(file: &mut File) -> Result<Arena<GroupID, Group>, TreePersistError> {
        let mut reader = IniReader::new(GROUPS_FILE.to_string(), file)?;
        let global_gen = reader.read_global_generation()?;

        let est = reader.lines.len().saturating_sub(3) / 4;
        let mut arena = Arena::with_preallocated_slots(est, global_gen);

        Self::load_groups_rec(&mut reader, &mut arena)?;

        Ok(arena)
    }

    fn load_groups_rec(
        reader: &mut IniReader,
        arena: &mut Arena<GroupID, Group>,
    ) -> Result<(), ContexedParseError> {
        let id = match reader.expect_section() {
            Ok(id) => GroupID::from_comb(id),
            Err(e) => {
                if matches!(e.inner, ParseError::UnexpectedEOF { .. }) {
                    return Ok(());
                }
                return Err(e);
            }
        };

        let name = reader.expect_field("name")?;

        let mut devices = HashSet::with_capacity_and_hasher(5, FxBuildHasher);
        for device_str in reader.expect_field("devices")?.split(',') {
            let device = device_str.parse::<u64>().map_err(|e| ContexedParseError {
                inner: ParseError::ParseInt(e),
                filename: reader.filename.clone(),
                line: reader.cur_line,
            })?;

            devices.insert(DeviceID::from_comb(device));
        }

        arena
            .insert_at(id, Group { id, name, devices })
            .map_err(|e| ContexedParseError {
                inner: ParseError::InsertAt(e),
                filename: reader.filename.clone(),
                line: reader.cur_line,
            })?;

        Self::load_groups_rec(reader, arena)
    }

    fn load_devices(file: &mut File) -> Result<Arena<DeviceID, Device>, TreePersistError> {
        let mut reader = IniReader::new(DEVICES_FILE.to_string(), file)?;
        let global_gen = reader.read_global_generation()?;

        let est = reader.lines.len().saturating_sub(3) / 4;
        let mut arena = Arena::with_preallocated_slots(est, global_gen);

        Self::load_devices_rec(&mut reader, &mut arena)?;

        Ok(arena)
    }

    fn load_devices_rec(
        reader: &mut IniReader,
        arena: &mut Arena<DeviceID, Device>,
    ) -> Result<(), ContexedParseError> {
        let id = match reader.expect_section() {
            Ok(id) => DeviceID::from_comb(id),
            Err(e) => {
                if matches!(e.inner, ParseError::UnexpectedEOF { .. }) {
                    return Ok(());
                }
                return Err(e);
            }
        };

        let name = reader.expect_field("name")?;
        let owner = reader.expect_field("owner")?;
        let owner = ExtensionID(owner);

        arena
            .insert_at(id, Device::new(id, name, owner))
            .map_err(|e| ContexedParseError {
                inner: ParseError::InsertAt(e),
                filename: reader.filename.clone(),
                line: reader.cur_line,
            })?;

        Self::load_devices_rec(reader, arena)
    }

    pub(super) fn save_groups(&mut self) -> Result<(), TreePersistError> {
        let w = &mut self.groups_writer;

        w.begin_write()?;
        w.write_header(self.groups.generation());

        for entry in self.groups.items().iter() {
            let Some(group) = entry.value() else {
                continue;
            };

            w.write_section(group.id().take());
            w.write_field("name", &group.name);

            w.buf.push_str("devices=");
            let mut first = true;
            for device_id in &group.devices {
                if !first {
                    w.buf.push(',');
                }
                w.buf.push_str(w.itoa_buf.format(device_id.take()));
                first = false;
            }
            w.buf.push('\n');
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

            w.write_section(device.id().take());
            w.write_field("name", &device.name);
            w.write_field("owner", &device.owner.0);
        }

        w.end_write()?;

        Ok(())
    }
}

pub struct IniWriter {
    file: Option<File>,
    buf: String,
    itoa_buf: itoa::Buffer,
}

impl IniWriter {
    pub fn new(file: File, capacity: usize) -> Self {
        Self {
            file: Some(file),
            buf: String::with_capacity(capacity),
            itoa_buf: itoa::Buffer::new(),
        }
    }

    pub fn fake() -> Self {
        Self {
            file: None,
            buf: String::with_capacity(1000),
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
        self.buf.push_str("generation=");
        self.buf.push_str(self.itoa_buf.format(generation));
        self.buf.push_str("\n\n");
    }

    pub fn write_section(&mut self, id: u64) {
        self.buf.push_str("\n[");
        self.buf.push_str(self.itoa_buf.format(id));
        self.buf.push_str("]\n");
    }

    pub fn write_field(&mut self, key: &str, value: &str) {
        self.buf.push_str(key);
        self.buf.push('=');
        self.buf.push_str(value);
        self.buf.push('\n');
    }
}

struct IniReader {
    filename: String,
    lines: Vec<String>,
    cur_line: usize,
}

enum IniLine<'a> {
    Section(u64),
    Field { key: &'a str, value: &'a str },
}

impl IniReader {
    fn new(filename: String, file: &mut File) -> io::Result<Self> {
        let content = io::read_to_string(file)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Ok(Self {
            filename,
            lines,
            cur_line: 0,
        })
    }

    fn next_line<'a>(&'a mut self) -> Result<Option<IniLine<'a>>, ContexedParseError> {
        loop {
            let Some(line) = self.lines.get(self.cur_line) else {
                return Ok(None);
            };
            self.cur_line += 1;

            let line = match line.find([';', '#']) {
                Some(pos) => &line[..pos].trim(),
                None => line.trim(),
            };

            if line.is_empty() {
                continue;
            }

            if let Some(stripped) = line.strip_prefix('[') {
                let section_str = stripped
                    .strip_suffix(']')
                    .ok_or_else(|| ContexedParseError {
                        filename: self.filename.clone(),
                        line: self.cur_line,
                        inner: ParseError::UnclosedSection,
                    })?;

                let section = section_str.parse::<u64>().map_err(|_| ContexedParseError {
                    filename: self.filename.clone(),
                    line: self.cur_line,
                    inner: ParseError::InvalidSectionNumber(section_str.to_string()),
                })?;

                return Ok(Some(IniLine::Section(section)));
            }

            return match line.split_once('=') {
                Some((key, value)) => Ok(Some(IniLine::Field {
                    key: key.trim(),
                    value: value.trim(),
                })),
                None => Err(ContexedParseError {
                    filename: self.filename.clone(),
                    line: self.cur_line,
                    inner: ParseError::InvalidLineFormat,
                }),
            };
        }
    }

    fn read_global_generation(&mut self) -> Result<u32, ContexedParseError> {
        let value = self.expect_field("generation")?;
        value.parse::<u32>().map_err(|e| ContexedParseError {
            filename: self.filename.clone(),
            line: self.cur_line,
            inner: ParseError::ParseInt(e),
        })
    }

    fn expect_section(&mut self) -> Result<u64, ContexedParseError> {
        match self.next_line()? {
            Some(IniLine::Section(id)) => Ok(id),
            Some(IniLine::Field { key, value }) => Err(ContexedParseError {
                inner: ParseError::UnexpectedField {
                    expected: "section".to_string(),
                    found_key: key.to_string(),
                    found_value: value.to_string(),
                },
                filename: self.filename.clone(),
                line: self.cur_line,
            }),
            None => Err(ContexedParseError {
                filename: self.filename.clone(),
                line: self.cur_line,
                inner: ParseError::UnexpectedEOF {
                    expected: "section".to_string(),
                },
            }),
        }
    }

    fn expect_field(&mut self, expected_key: &str) -> Result<String, ContexedParseError> {
        match self.next_line()? {
            Some(IniLine::Field { key, value }) if key == expected_key => Ok(value.to_string()),
            Some(IniLine::Field { key, value }) => Err(ContexedParseError {
                inner: ParseError::UnexpectedField {
                    expected: format!("field '{}'", expected_key),
                    found_key: key.to_string(),
                    found_value: value.to_string(),
                },
                filename: self.filename.clone(),
                line: self.cur_line,
            }),
            Some(IniLine::Section(id)) => Err(ContexedParseError {
                filename: self.filename.clone(),
                line: self.cur_line,
                inner: ParseError::UnexpectedSection {
                    expected: format!("field '{}'", expected_key),
                    found: id,
                },
            }),
            None => Err(ContexedParseError {
                filename: self.filename.clone(),
                line: self.cur_line,
                inner: ParseError::UnexpectedEOF {
                    expected: format!("field '{}'", expected_key),
                },
            }),
        }
    }
}

// TODO add testing!!

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::glacier::tree::Zone;

#[derive(Error, Debug)]
pub enum ZonesFileError {
    #[error("Invalid header syntax on line {line}: '{content}'")]
    InvalidHeaderSyntax { line: usize, content: String },
    #[error("Unexpected line outside of zone section on line {line}: '{content}'")]
    UnexpectedLineOutsideZone { line: usize, content: String },
    #[error("Missing '=' delimiter on line {line}{}: '{content}'", 
        zone_context(.zone_id, .zone_start_line))]
    MissingEqualsDelimiter {
        line: usize,
        content: String,
        zone_id: Option<String>,
        zone_start_line: Option<usize>,
    },
    #[error(
        "Device missing '.' delimiter on line {line} in zone [{zone_id}] (starting at line {zone_start_line}): '{content}'"
    )]
    MissingDotDelimiter {
        line: usize,
        content: String,
        zone_id: String,
        zone_start_line: usize,
    },
    #[error(
        "Invalid key '{key}' on line {line} in zone [{zone_id}] (starting at line {zone_start_line}): '{content}'"
    )]
    InvalidKey {
        line: usize,
        key: String,
        content: String,
        zone_id: String,
        zone_start_line: usize,
    },
    #[error(
        "Invalid boolean value '{value}' on line {line} in zone [{zone_id}] (starting at line {zone_start_line}): '{content}'"
    )]
    InvalidBooleanValue {
        line: usize,
        value: String,
        content: String,
        zone_id: String,
        zone_start_line: usize,
    },
    #[error("Zone [{zone_id}] missing name (zone spans lines {zone_start_line}-{zone_end_line})")]
    ZoneMissingName {
        zone_id: String,
        zone_start_line: usize,
        zone_end_line: usize,
    },
    #[error("Zone [{zone_id}] not found in file")]
    ZoneNotFound { zone_id: String },
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Two zones exist under the same ID: '{zone_id}'")]
    DuplicateZones { zone_id: String },
}

fn zone_context(zone_id: &Option<String>, zone_start_line: &Option<usize>) -> String {
    match (zone_id, zone_start_line) {
        (Some(id), Some(start)) => format!(" in zone [{id}] (starting at line {start})"),
        _ => String::new(),
    }
}

#[derive(Error, Debug)]
pub enum DevicesFileError {
    #[error("Missing '=' delimiter on line {line}: '{content}'")]
    MissingEqualsDelimiter { line: usize, content: String },
    #[error("Missing '.' delimiter on line {line}: '{content}'")]
    MissingDotDelimiter { line: usize, content: String },
    #[error("Device {floe_id}.{device_id} not found in file")]
    DeviceNotFound { floe_id: String, device_id: String },
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Two devices exist under the same ID: '{floe_id}.{device_id}'")]
    DuplicateDevices { floe_id: String, device_id: String },
}

pub fn parse_zones_file(
    content: String,
) -> Result<(Vec<Zone>, FxHashMap<String, u16>), ZonesFileError> {
    let mut zones = Vec::with_capacity(10);
    let mut zone_idx_lut = FxHashMap::default();

    let mut in_zone = false;
    let mut cur_zone_id: Option<String> = None;
    let mut cur_zone_name = None;
    let mut cur_zone_devs = FxHashSet::default();
    let mut cur_zone_disabled = false;
    let mut cur_zone_start_line: Option<usize> = None;

    for (line_num, line) in content.lines().enumerate() {
        let line_number = line_num + 1;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if in_zone {
                let zone_id = cur_zone_id.clone().unwrap();
                let zone_start = cur_zone_start_line.unwrap();

                zones.push(Zone {
                    name: cur_zone_name
                        .take()
                        .ok_or(ZonesFileError::ZoneMissingName {
                            zone_id: zone_id.clone(),
                            zone_start_line: zone_start,
                            zone_end_line: line_number - 1,
                        })?,
                    devices: cur_zone_devs.clone(),
                    disabled: cur_zone_disabled,
                    idxs: SmallVec::new(),
                });
                cur_zone_devs.clear();
                cur_zone_disabled = false;
            }

            if !line.ends_with(']') {
                return Err(ZonesFileError::InvalidHeaderSyntax {
                    line: line_number,
                    content: line.to_string(),
                });
            }

            let zone_id = line[1..line.len() - 1].to_string();
            let res = zone_idx_lut.insert(zone_id.clone(), zones.len() as u16);
            if res.is_some() {
                return Err(ZonesFileError::DuplicateZones { zone_id });
            }

            cur_zone_id = Some(zone_id);
            cur_zone_start_line = Some(line_number);
            in_zone = true;
        } else {
            if !in_zone {
                return Err(ZonesFileError::UnexpectedLineOutsideZone {
                    line: line_number,
                    content: line.to_string(),
                });
            }

            let Some((lhs, rhs)) = line.split_once('=') else {
                return Err(ZonesFileError::MissingEqualsDelimiter {
                    line: line_number,
                    content: line.to_string(),
                    zone_id: cur_zone_id.clone(),
                    zone_start_line: cur_zone_start_line,
                });
            };

            let key = lhs.trim();
            let zone_id = cur_zone_id.as_ref().unwrap();
            let zone_start = cur_zone_start_line.unwrap();

            match key {
                "name" => {
                    cur_zone_name = Some(rhs.trim().to_string());
                }
                "device" => {
                    let Some((floe_id, device_id)) = rhs.trim().split_once('.') else {
                        return Err(ZonesFileError::MissingDotDelimiter {
                            line: line_number,
                            content: line.to_string(),
                            zone_id: zone_id.clone(),
                            zone_start_line: zone_start,
                        });
                    };
                    cur_zone_devs.insert((floe_id.to_string(), device_id.to_string()));
                }
                "disabled" => {
                    cur_zone_disabled = match rhs.trim() {
                        "true" => true,
                        "false" => false,
                        val => {
                            return Err(ZonesFileError::InvalidBooleanValue {
                                line: line_number,
                                value: val.to_string(),
                                content: line.to_string(),
                                zone_id: zone_id.clone(),
                                zone_start_line: zone_start,
                            });
                        }
                    }
                }
                _ => {
                    return Err(ZonesFileError::InvalidKey {
                        line: line_number,
                        key: key.to_string(),
                        content: line.to_string(),
                        zone_id: zone_id.clone(),
                        zone_start_line: zone_start,
                    });
                }
            }
        }
    }

    // add final zone
    if in_zone {
        let zone_id = cur_zone_id.unwrap();
        let zone_start = cur_zone_start_line.unwrap();
        let zone_end = content.lines().count();

        zones.push(Zone {
            name: cur_zone_name
                .take()
                .ok_or(ZonesFileError::ZoneMissingName {
                    zone_id: zone_id.clone(),
                    zone_start_line: zone_start,
                    zone_end_line: zone_end,
                })?,
            devices: cur_zone_devs.clone(),
            disabled: cur_zone_disabled,
            idxs: SmallVec::new(),
        });
    }

    Ok((zones, zone_idx_lut))
}

/// Appends zone to end of file
pub async fn add_zone(path: &str, zone_id: &str, zone: &Zone) -> Result<(), ZonesFileError> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;

    let mut content = format!("\n[{}]\n", zone_id);
    content.push_str(&format!("name = {}\n", zone.name));

    for (floe_id, device_id) in &zone.devices {
        content.push_str(&format!("device = {}.{}\n", floe_id, device_id));
    }

    if zone.disabled {
        content.push_str("disabled = true\n");
    }

    file.write_all(content.as_bytes()).await?;

    Ok(())
}

/// Opens file, finds section from `zone_id`,
/// deletes old data and rewrites with `new_zone`
pub async fn modify_zone(path: &str, zone_id: &str, new_zone: &Zone) -> Result<(), ZonesFileError> {
    let content = fs::read_to_string(path).await?;
    let lines: Vec<&str> = content.lines().collect();

    let mut result = String::new();
    let mut found = false;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let current_zone_id = &trimmed[1..trimmed.len() - 1];

            if current_zone_id == zone_id {
                found = true;

                result.push_str(&format!("[{}]\n", zone_id));
                result.push_str(&format!("name = {}\n", new_zone.name));

                for (floe_id, device_id) in &new_zone.devices {
                    result.push_str(&format!("device = {}.{}\n", floe_id, device_id));
                }

                if new_zone.disabled {
                    result.push_str("disabled = true\n");
                }

                i += 1;
                while i < lines.len() {
                    let next_trimmed = lines[i].trim();
                    if next_trimmed.starts_with('[') && next_trimmed.ends_with(']') {
                        result.push('\n');
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }

        result.push_str(line);
        result.push('\n');
        i += 1;
    }

    if !found {
        return Err(ZonesFileError::ZoneNotFound {
            zone_id: zone_id.to_string(),
        });
    }

    fs::write(path, result).await?;

    Ok(())
}

pub fn parse_devices_file(
    content: String,
) -> Result<FxHashMap<String, FxHashMap<String, String>>, DevicesFileError> {
    let mut map: FxHashMap<String, FxHashMap<String, String>> = FxHashMap::default();

    for (line_num, line) in content.lines().enumerate() {
        let line_number = line_num + 1;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((lhs, name)) = line.split_once('=') else {
            return Err(DevicesFileError::MissingEqualsDelimiter {
                line: line_number,
                content: line.to_string(),
            });
        };

        let Some((floe_id, device_id)) = lhs.trim().split_once('.') else {
            return Err(DevicesFileError::MissingDotDelimiter {
                line: line_number,
                content: line.to_string(),
            });
        };

        match map.get_mut(floe_id) {
            Some(floe_map) => {
                let res = floe_map.insert(device_id.to_string(), name.trim().to_string());

                if res.is_some() {
                    return Err(DevicesFileError::DuplicateDevices {
                        floe_id: floe_id.to_string(),
                        device_id: device_id.to_string(),
                    });
                }
            }
            None => {
                let mut floe_map = FxHashMap::default();
                floe_map.insert(device_id.to_string(), name.trim().to_string());
                map.insert(floe_id.to_string(), floe_map);
            }
        }
    }

    Ok(map)
}

/// Appends new device name to end of file
pub async fn add_device(
    path: &str,
    floe_id: &str,
    device_id: &str,
    name: &str,
) -> Result<(), DevicesFileError> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;

    let content = format!("{}.{} = {}\n", floe_id, device_id, name);
    file.write_all(content.as_bytes()).await?;

    Ok(())
}

/// Opens file, finds location of device line
/// and replaces it with new name
pub async fn rename_device(
    path: &str,
    floe_id: &str,
    device_id: &str,
    new_name: &str,
) -> Result<(), DevicesFileError> {
    let content = fs::read_to_string(path).await?;

    let mut result = String::new();
    let mut found = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if let Some((lhs, _)) = trimmed.split_once('=')
            && let Some((file_floe_id, file_device_id)) = lhs.trim().split_once('.')
            && file_floe_id == floe_id
            && file_device_id == device_id
        {
            found = true;
            result.push_str(&format!("{}.{} = {}\n", floe_id, device_id, new_name));
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if !found {
        return Err(DevicesFileError::DeviceNotFound {
            floe_id: floe_id.to_string(),
            device_id: device_id.to_string(),
        });
    }

    fs::write(path, result).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashMap;

    use crate::glacier::file::parse_devices_file;

    #[test]
    fn test_parse_devices_file() {
        let content = r#"

    
ESPHome.0199a2c3-0ed1-7665-9d18-8c81901f8e5d = Kitchen Pantry
ESPHome.0199a2c3-58b4-76a9-9193-8f13beafcbe9 = Bar A 
ESPHome.0199a2c3-4921-75b5-b7ca-205a00f5d03f = Kitchen Sink
ESPHome.0199a2c3-85e4-77dc-97ae-ca5feb8735fe = Living B
ESPHome.0199a2c3-7811-72a2-b7ad-3c4124d2abf1 = Living A
        
    "#;

        let actual = parse_devices_file(content.to_string()).unwrap();

        let mut expected = FxHashMap::default();
        let mut esphome_devices = FxHashMap::default();

        esphome_devices.insert(
            "0199a2c3-0ed1-7665-9d18-8c81901f8e5d".to_string(),
            "Kitchen Pantry".to_string(),
        );
        esphome_devices.insert(
            "0199a2c3-58b4-76a9-9193-8f13beafcbe9".to_string(),
            "Bar A".to_string(),
        );
        esphome_devices.insert(
            "0199a2c3-4921-75b5-b7ca-205a00f5d03f".to_string(),
            "Kitchen Sink".to_string(),
        );
        esphome_devices.insert(
            "0199a2c3-85e4-77dc-97ae-ca5feb8735fe".to_string(),
            "Living B".to_string(),
        );
        esphome_devices.insert(
            "0199a2c3-7811-72a2-b7ad-3c4124d2abf1".to_string(),
            "Living A".to_string(),
        );

        expected.insert("ESPHome".to_string(), esphome_devices);

        assert_eq!(actual, expected);

        assert!(parse_devices_file("asdf = asdf".to_string()).is_err());
    }

    #[test]
    fn test_parse_zones_file() {
        use super::{Zone, parse_zones_file};
        use rustc_hash::FxHashSet;
        use smallvec::SmallVec;

        let content = r#"
# comment
[550e8400-e29b-41d4-a716-446655440000]
name = Living Room
device = ESPHome.0199a2c3-0ed1-7665-9d18-8c81901f8e5d
device = ESPHome.0199a2c3-4921-75b5-b7ca-205a00f5d03f

[660e8400-e29b-41d4-a716-446655440001]
name = Kitchen
device = ESPHome.0199a2c3-58b4-76a9-9193-8f13beafcbe9
disabled = true
    "#;

        let (actual_zones, actual_idx_lut) = parse_zones_file(content.to_string()).unwrap();

        let mut expected_zones = Vec::new();

        let mut zone1_devices = FxHashSet::default();
        zone1_devices.insert((
            "ESPHome".to_string(),
            "0199a2c3-0ed1-7665-9d18-8c81901f8e5d".to_string(),
        ));
        zone1_devices.insert((
            "ESPHome".to_string(),
            "0199a2c3-4921-75b5-b7ca-205a00f5d03f".to_string(),
        ));

        expected_zones.push(Zone {
            name: "Living Room".to_string(),
            disabled: false,
            devices: zone1_devices,
            idxs: SmallVec::new(),
        });

        let mut zone2_devices = FxHashSet::default();
        zone2_devices.insert((
            "ESPHome".to_string(),
            "0199a2c3-58b4-76a9-9193-8f13beafcbe9".to_string(),
        ));

        expected_zones.push(Zone {
            name: "Kitchen".to_string(),
            disabled: true,
            devices: zone2_devices,
            idxs: SmallVec::new(),
        });

        let mut expected_idx_lut = FxHashMap::default();
        expected_idx_lut.insert("550e8400-e29b-41d4-a716-446655440000".to_string(), 0);
        expected_idx_lut.insert("660e8400-e29b-41d4-a716-446655440001".to_string(), 1);

        assert_eq!(actual_zones.len(), expected_zones.len());
        for (actual, expected) in actual_zones.iter().zip(expected_zones.iter()) {
            assert_eq!(actual.name, expected.name);
            assert_eq!(actual.disabled, expected.disabled);
            assert_eq!(actual.devices, expected.devices);
        }
        assert_eq!(actual_idx_lut, expected_idx_lut);

        assert!(parse_zones_file("[invalid\nname = test".to_string()).is_err());
        assert!(parse_zones_file("name = test".to_string()).is_err());
    }
}

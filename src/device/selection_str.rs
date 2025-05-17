use super::error::DeviceSelectorError;

/// Converts a selection string into its parts
///  all -> All
///  kitchen -> Zone("kitchen")
///  kitchen.sink -> Device("kitchen", "sink")
///  kitchen.sink.light_bulb -> Entity("kitchen", "sink", "light_bulb")
pub enum SelectionString<'a> {
    All,
    /// zone_name
    Zone(&'a str),
    /// zone_name, dev_name
    Device(&'a str, &'a str),
    /// zone_name, dev_name, entity_name
    Entity(&'a str, &'a str, &'a str),
}

impl<'a> SelectionString<'a> {
    pub fn new(selection_str: &'a str) -> Result<Self, DeviceSelectorError> {
        if selection_str == "all" {
            return Ok(Self::All);
        }

        let parts: Vec<&str> = selection_str.split(".").collect();
        if parts.len() < 1 || parts.len() > 3 {
            return Err(DeviceSelectorError::BadSelector);
        }

        let zone_name = parts.get(0).unwrap();

        if let Some(dev_name) = parts.get(1) {
            if let Some(entity_name) = parts.get(2) {
                Ok(Self::Entity(zone_name, dev_name, entity_name))
            } else {
                Ok(Self::Device(zone_name, dev_name))
            }
        } else {
            Ok(Self::Zone(zone_name))
        }
    }

    pub fn get_zone_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(zone_name) => Some(zone_name),
            Self::Device(zone_name, _) => Some(zone_name),
            Self::Entity(zone_name, _, _) => Some(zone_name),
        }
    }

    pub fn get_dev_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(..) => None,
            Self::Device(_, dev_name) => Some(dev_name),
            Self::Entity(_, dev_name, _) => Some(dev_name),
        }
    }

    pub fn get_entity_name(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Zone(..) => None,
            Self::Device(_, _) => None,
            Self::Entity(_, _, entity_name) => Some(&entity_name),
        }
    }
}

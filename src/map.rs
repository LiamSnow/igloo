use std::{collections::HashMap, error::Error, sync::Arc};

use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{cli::model::SwitchState, command::{LightState, RackSubdeviceCommand, SubdeviceState, SubdeviceStateUpdate}, config::{IglooConfig, UIElementConfig, ZonesConfig}, providers::{esphome, DeviceConfig}, selector::SelectionString};

pub struct IglooStack {
    pub channels: DeviceCmdChannelMap,
    pub ui_elements: HashMap<String, Vec<UIElement>>
}

pub type DeviceCmdChannelMap = HashMap<String, HashMap<String, Sender<RackSubdeviceCommand>>>;
pub type DeviceIDMap = HashMap<String, HashMap<String, usize>>; //maps Zone, Device -> ID
// pub type StatesMap = Vec<HashMap<u32, SubdeviceState>>; //maps device ID -> HashTable<SubdeviceKey, State>

#[derive(Default)]
pub struct SubdeviceStates {
    light: Vec<HashMap<String, LightState>>,
    switch: Vec<HashMap<String, SwitchState>>,
}

#[derive(Clone, Serialize)]
pub struct UIElement {
    pub cfg: UIElementConfig,
    pub id: usize
}

pub struct UIElementState {
    pub id: usize,
    pub state: SubdeviceState
}

#[derive(Default, Clone)]
pub struct UIElementStates {
    pub light: Vec<Option<LightState>>,
    pub switch: Vec<Option<SwitchState>>,
}

#[derive(Default)]
struct UIElementSelections {
    light: Vec<UIElementSelection>,
    switch: Vec<UIElementSelection>
}

pub enum UIElementSelection {
    All,
    Zone(Vec<usize>),
    Device(usize),
    Subdevice(usize, String)
}

impl UIElementSelection {
    fn new(id_map: &DeviceIDMap, cfg: &UIElementConfig) -> Result<Self, Box<dyn Error>> {
        Ok(match SelectionString::new(cfg.get_selector_str())? {
            SelectionString::All => Self::All,
            SelectionString::Zone(zone_name) => {
                let zone = id_map.get(zone_name).unwrap(); //FIXME
                Self::Zone(zone.values().copied().collect())
            },
            SelectionString::Device(zone_name, dev_name) => {
                let zone = id_map.get(zone_name).unwrap(); //FIXME
                let dev = zone.get(dev_name).unwrap();
                Self::Device(*dev)
            },
            SelectionString::Subdevice(zone_name, dev_name, sub_name) => {
                let zone = id_map.get(zone_name).unwrap(); //FIXME
                let dev = zone.get(dev_name).unwrap();
                Self::Subdevice(*dev, sub_name.to_string())
            },
        })
    }
}

impl IglooStack {
    pub async fn init(config: IglooConfig) -> Result<(Arc<Self>, Receiver<UIElementStates>), Box<dyn Error>> {
        let (map, mut dev_states, dev_id_lut, mut dev_update_rx) = Self::make_map(config.zones)?;

        let mut ui = HashMap::new();
        let mut ui_selections = UIElementSelections { ..Default::default() };
        let mut ui_states = UIElementStates { ..Default::default() };
        let (mut light_id, mut switch_id) = (0, 0);
        for (group_name, cfgs) in config.ui {
            let mut group_els = Vec::new();
            for cfg in cfgs {
                let sel = UIElementSelection::new(&dev_id_lut, &cfg)?;
                match cfg {
                    UIElementConfig::Light(_) => {
                        ui_selections.light.push(sel);
                        ui_states.light.push(None);
                        group_els.push(UIElement { cfg, id: light_id });
                        light_id += 1;
                    }
                    UIElementConfig::Switch(_) => {
                        ui_selections.switch.push(sel);
                        ui_states.switch.push(None);
                        group_els.push(UIElement { cfg, id: switch_id });
                        switch_id += 1;
                    },
                }
            }
            ui.insert(group_name, group_els);
        }

        let (ui_tx, ui_rx) = mpsc::channel::<UIElementStates>(5);
        tokio::spawn(async move {
            while let Some(update) = dev_update_rx.recv().await {
                match update.value {
                    SubdeviceState::Light(new_state) => {
                        //save new state to device states
                        dev_states.light.get_mut(update.dev_id).unwrap() //FIXME
                            .insert(update.subdev_name.clone(), new_state.clone());

                        //apple changes to effected elements
                        for (id, selection) in ui_selections.light.iter().enumerate() {
                            match selection {
                                UIElementSelection::All => {
                                    let mut light_states = Vec::new();
                                    for map in &dev_states.light {
                                        for (_, state) in map {
                                            light_states.push(state);
                                        }
                                    }
                                    let avg_light_state = LightState::avg(light_states);
                                    *ui_states.light.get_mut(id).unwrap() = Some(avg_light_state); //FIXME
                                },
                                UIElementSelection::Zone(dids) => if dids.contains(&id) {
                                    let mut light_states = Vec::new();
                                    for did in dids {
                                        for (_, state) in dev_states.light.get(*did).unwrap() { //FIXME
                                            light_states.push(state);
                                        }
                                    }
                                    let avg_light_state = LightState::avg(light_states);
                                    *ui_states.light.get_mut(id).unwrap() = Some(avg_light_state); //FIXME
                                },
                                UIElementSelection::Device(did) => if *did == id {
                                    let mut light_states = Vec::new();
                                    for (_, state) in dev_states.light.get(*did).unwrap() { //FIXME
                                        light_states.push(state);
                                    }
                                    let avg_light_state = LightState::avg(light_states);
                                    *ui_states.light.get_mut(id).unwrap() = Some(avg_light_state); //FIXME
                                },
                                UIElementSelection::Subdevice(did, subdev_name) => if *did == id && *subdev_name == update.subdev_name {
                                    *ui_states.light.get_mut(id).unwrap() = Some(new_state.clone()); //FIXME
                                },
                            }
                        }
                    },
                    SubdeviceState::Switch(_) => todo!(),
                }
                ui_tx.send(ui_states.clone()).await.unwrap(); //FIXME
            }
        });

        Ok((Arc::new(IglooStack { channels: map, ui_elements: ui }), ui_rx))
    }

    fn make_map(zones: ZonesConfig) -> Result<(DeviceCmdChannelMap, SubdeviceStates, DeviceIDMap, Receiver<SubdeviceStateUpdate>), Box<dyn Error>> {
        let (update_tx, update_rx) = mpsc::channel::<SubdeviceStateUpdate>(50);

        let mut map = HashMap::new();
        let mut id_map = HashMap::new();
        let mut states = SubdeviceStates { ..Default::default() };
        let mut dev_id = 0;

        for (zone_name, devs) in zones {
            let mut zone_map = HashMap::new();
            let mut zone_id_map = HashMap::new();
            for (dev_name, dev_cfg) in devs {
                let tx = match dev_cfg {
                    DeviceConfig::ESPHome(cfg) => esphome::new(cfg, dev_id, update_tx.clone())?,
                    DeviceConfig::HomeKit(_cfg) => todo!(),
                };
                zone_map.insert(dev_name.clone(), tx);
                zone_id_map.insert(dev_name.clone(), dev_id);
                states.light.push(HashMap::new());
                states.switch.push(HashMap::new());
                dev_id += 1;
            }
            id_map.insert(zone_name.clone(), zone_id_map);
            map.insert(zone_name.clone(), zone_map);
        }
        Ok((map, states, id_map, update_rx))
    }

}



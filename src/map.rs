use std::{collections::HashMap, error::Error, sync::Arc};

use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{cli::model::SwitchState, command::{LightState, RackSubdeviceCommand, SubdeviceState, SubdeviceStateUpdate}, config::{IglooConfig, UIElementConfig, ZonesConfig}, providers::{esphome, DeviceConfig}, selector::SelectionString};

pub struct IglooStack {
    pub channels: DeviceCmdChannelMap,
    pub ui_group_names: Vec<String>,
    pub ui_elements: Vec<UIElement>
}

pub type DeviceCmdChannelMap = HashMap<String, HashMap<String, Sender<RackSubdeviceCommand>>>;
pub type DeviceIDMap = HashMap<String, HashMap<String, usize>>; //maps Zone, Device -> ID
// pub type StatesMap = Vec<HashMap<u32, SubdeviceState>>; //maps device ID -> HashTable<SubdeviceKey, State>

#[derive(Default)]
pub struct SubdeviceStates {
    /// did.sub_name -> state
    light: Vec<HashMap<String, LightState>>,
    switch: Vec<HashMap<String, SwitchState>>,
}

#[derive(Clone, Serialize)]
pub struct UIElement {
    pub cfg: UIElementConfig,
    pub gid: usize,
    pub eid: usize
}

#[derive(Default)]
struct UIElementEffection {
    light: UIElementEffectionGroup,
    switch: UIElementEffectionGroup
}

#[derive(Default)]
pub struct UIElementEffectionGroup {
    /// eid
    all: usize,
    /// did -> eid
    dev: HashMap<usize, usize>,
    /// did -> zid
    zone: HashMap<usize, Vec<usize>>,
    /// (did,sub_name) -> eid
    sub: HashMap<(usize, String), usize>
}

impl IglooStack {
    pub async fn init(config: IglooConfig) -> Result<(Arc<Self>, Receiver<Vec<Option<SubdeviceState>>>), Box<dyn Error>> {
        let (map, mut dev_states, dev_id_lut, mut dev_update_rx) = Self::make_map(config.zones)?;

        let mut gid = 0;
        let mut eid = 0;
        let mut ui_group_names = Vec::new();
        let mut ui_elements = Vec::new();
        let mut ui_states: Vec<Option<SubdeviceState>> = Vec::new();
        let mut ui_effections = UIElementEffection { ..Default::default() };
        let mut zone_lut: Vec<(usize, Vec<usize>)> = Vec::new(); //zid -> (eid, Vec<did>)
        let mut zid = 0;
        for (group_name, cfgs) in config.ui {
            for cfg in cfgs {
                let effect_group = match cfg {
                    UIElementConfig::Light(_) => &mut ui_effections.light,
                    UIElementConfig::Switch(_) => &mut ui_effections.switch,
                };

                match SelectionString::new(cfg.get_selector_str())? {
                    SelectionString::All => effect_group.all = eid,
                    SelectionString::Zone(zone_name) => {
                        let zone = dev_id_lut.get(zone_name).unwrap().values(); //FIXME
                        let mut dids = Vec::new();
                        for did in zone {
                            let v = effect_group.zone.entry(*did).or_insert(Vec::new());
                            v.push(zid);
                            dids.push(*did);
                        }
                        zone_lut.push((eid, dids));
                        zid += 1;
                    },
                    SelectionString::Device(zone_name, dev_name) => {
                        let did = dev_id_lut.get(zone_name).unwrap().get(dev_name).unwrap(); //FIXME
                        effect_group.dev.insert(*did, eid);
                    },
                    SelectionString::Subdevice(zone_name, dev_name, sub_name) => {
                        let did = dev_id_lut.get(zone_name).unwrap().get(dev_name).unwrap(); //FIXME
                        effect_group.sub.insert((*did, sub_name.to_string()), eid);
                    },
                }

                ui_elements.push(UIElement { cfg, gid, eid, });
                ui_states.push(None);
                eid += 1;
            }
            ui_group_names.push(group_name);
            gid += 1;
        }

        let (ui_tx, ui_rx) = mpsc::channel(5);
        tokio::spawn(async move {
            while let Some(update) = dev_update_rx.recv().await {
                match update.value {
                    SubdeviceState::Light(new_state) => {
                        //save new state to device states
                        dev_states.light.get_mut(update.dev_id).unwrap() //FIXME
                            .insert(update.subdev_name.clone(), new_state.clone());

                        //apply changes to effected elements

                        //subdevice
                        if let Some(eid) = ui_effections.light.sub.get(&(update.dev_id, update.subdev_name)) {
                            let ui_state = ui_states.get_mut(*eid).unwrap(); //FIXME
                            *ui_state = Some(SubdeviceState::Light(new_state.clone()));
                        }

                        //device
                        if let Some(eid) = ui_effections.light.dev.get(&update.dev_id) {
                            //average state of light subdevices
                            let mut light_states = Vec::new();
                            let map = dev_states.light.get(update.dev_id).unwrap(); //FIXME
                            for (_, state) in map {
                                light_states.push(state);
                            }
                            let avg_light_state = LightState::avg(light_states);

                            //apply
                            let ui_state = ui_states.get_mut(*eid).unwrap(); //FIXME
                            *ui_state = Some(SubdeviceState::Light(avg_light_state.clone()));
                        }

                        //zone
                        if let Some(zids) = ui_effections.light.zone.get(&update.dev_id) {
                            for zid in zids {
                                let (eid, dids) = zone_lut.get(*zid).unwrap(); //FIXME
                                let mut light_states = Vec::new();
                                for did in dids {
                                    let map = dev_states.light.get(*did).unwrap(); //FIXME
                                    for (_, state) in map {
                                        light_states.push(state);
                                    }
                                }
                                let avg_light_state = LightState::avg(light_states);
                                let ui_state = ui_states.get_mut(*eid).unwrap(); //FIXME
                                *ui_state = Some(SubdeviceState::Light(avg_light_state.clone()));
                            }
                        }

                        //all
                        let mut light_states = Vec::new();
                        for map in &dev_states.light {
                            for (_, state) in map {
                                light_states.push(state);
                            }
                        }
                        let avg_light_state = LightState::avg(light_states);
                        let ui_state = ui_states.get_mut(ui_effections.light.all).unwrap(); //FIXME
                        *ui_state = Some(SubdeviceState::Light(avg_light_state.clone()));
                    },
                    SubdeviceState::Switch(_) => todo!(),
                }
                ui_tx.send(ui_states.clone()).await.unwrap(); //FIXME
            }
        });

        Ok((Arc::new(IglooStack { channels: map, ui_group_names, ui_elements }), ui_rx))
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



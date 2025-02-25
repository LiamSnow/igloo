use std::{collections::HashMap, error::Error, sync::Arc};

use serde::Serialize;
use tokio::sync::{mpsc::{self, Receiver, Sender}, oneshot, Mutex};

use crate::{cli::model::SwitchState, command::{LightState, RackSubdeviceCommand, SubdeviceState, SubdeviceStateUpdate}, config::{IglooConfig, UIElementConfig, ZonesConfig}, providers::{esphome, DeviceConfig}, selector::{OwnedSelection, Selection}};

pub struct IglooStack {
    /// Maps zone.device -> device command channel
    pub cmd_chan_map: DeviceCommandChannelMap,
    pub ui_elements: HashMap<String, Vec<UIElement>>,
    pub current_effects: Mutex<Vec<(OwnedSelection, oneshot::Sender::<bool>)>>
}

/// Maps zone.device -> device command channel
pub type DeviceCommandChannelMap = HashMap<String, HashMap<String, Sender<RackSubdeviceCommand>>>;

#[derive(Serialize)]
pub struct UIElement {
    pub cfg: UIElementConfig,
    pub eid: usize
}

#[derive(Default)]
/// Maps device_id.subdevice_name -> state
pub struct SubdeviceStates {
    light: Vec<HashMap<String, LightState>>,
    switch: Vec<HashMap<String, SwitchState>>,
}

#[derive(Default)]
///Maps device ID (did) -> element ID (eid)'s that are effected by its changes
struct DeviceDependencyMaps {
    light: DeviceDependencyMap,
    switch: DeviceDependencyMap
}

#[derive(Default)]
struct DeviceDependencyMap {
    /// eid
    all: usize,
    /// did -> eid
    dev: HashMap<usize, usize>,
    /// did -> zid
    zone: HashMap<usize, Vec<usize>>,
    /// (did,sub_name) -> eid
    subdev: HashMap<(usize, String), usize>
}

impl IglooStack {
    pub async fn init(config: IglooConfig) -> Result<(Arc<Self>, Receiver<Vec<Option<SubdeviceState>>>), Box<dyn Error>> {
        let (cmd_chan_map, dev_states, dev_id_lut, dev_update_rx) = Self::make_map(config.zones)?;
        let (ui_elements, element_states, dev_effections, zone_lut) = Self::make_ui(config.ui, dev_id_lut)?;
        let (ui_update_tx, ui_update_rx) = mpsc::channel(5);
        tokio::spawn(Self::ui_update_thread(dev_update_rx, dev_states, element_states, dev_effections, zone_lut, ui_update_tx));
        let current_effects = Mutex::new(Vec::new());
        Ok((Arc::new(IglooStack { cmd_chan_map, ui_elements, current_effects }), ui_update_rx))
    }

    fn make_map(zones: ZonesConfig) -> Result<(DeviceCommandChannelMap, SubdeviceStates, HashMap<String, HashMap<String, usize>>, Receiver<SubdeviceStateUpdate>), Box<dyn Error>> {
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

    fn make_ui(ui_cfg: HashMap<String, Vec<UIElementConfig>>, dev_id_lut: HashMap<String, HashMap<String, usize>>) -> Result<(
        HashMap<String, Vec<UIElement>>, Vec<Option<SubdeviceState>>, DeviceDependencyMaps, Vec<(usize, Vec<usize>)>), Box<dyn Error>> {
        let (mut eid, mut zid) = (0, 0);
        let mut ui = HashMap::new();
        let mut element_states = Vec::new();
        let mut dep_map = DeviceDependencyMaps { ..Default::default() };
        let mut zone_lut = Vec::new(); //zid -> (eid, Vec<did>)
        for (group_name, cfgs) in ui_cfg {
            let mut group_elements = Vec::new();

            for cfg in cfgs {
                let (sel_str, effect_group) = match cfg {
                    UIElementConfig::Light(ref sel_str, ..) => (sel_str, &mut dep_map.light),
                    UIElementConfig::Switch(ref sel_str) => (sel_str, &mut dep_map.switch),
                    _ => continue //FIXME add element
                };

                match Selection::new(sel_str)? {
                    Selection::All => effect_group.all = eid,
                    Selection::Zone(zone_name) => {
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
                    Selection::Device(zone_name, dev_name) => {
                        let did = dev_id_lut.get(zone_name).unwrap().get(dev_name).unwrap(); //FIXME
                        effect_group.dev.insert(*did, eid);
                    },
                    Selection::Subdevice(zone_name, dev_name, sub_name) => {
                        let did = dev_id_lut.get(zone_name).unwrap().get(dev_name).unwrap(); //FIXME
                        effect_group.subdev.insert((*did, sub_name.to_string()), eid);
                    },
                }

                group_elements.push(UIElement { cfg, eid, });
                element_states.push(None);
                eid += 1;
            }

            ui.insert(group_name, group_elements);
        }

        Ok((ui, element_states, dep_map, zone_lut))
    }

    async fn ui_update_thread(mut dev_update_rx: Receiver<SubdeviceStateUpdate>, mut dev_states: SubdeviceStates, mut element_states: Vec<Option<SubdeviceState>>,
        dep_map: DeviceDependencyMaps, zone_lut: Vec<(usize, Vec<usize>)>, ui_update_tx: Sender<Vec<Option<SubdeviceState>>>) {
        while let Some(update) = dev_update_rx.recv().await {
            match update.value {
                SubdeviceState::Light(new_state) => {
                    //save new state to device states
                    dev_states.light.get_mut(update.dev_id).unwrap() //FIXME
                        .insert(update.subdev_name.clone(), new_state.clone());

                    //apply changes to effected elements

                    //subdevice
                    if let Some(eid) = dep_map.light.subdev.get(&(update.dev_id, update.subdev_name)) {
                        let ui_state = element_states.get_mut(*eid).unwrap(); //FIXME
                        *ui_state = Some(SubdeviceState::Light(new_state.clone()));
                    }

                    //device
                    if let Some(eid) = dep_map.light.dev.get(&update.dev_id) {
                        //average state of light subdevices
                        let mut light_states = Vec::new();
                        let map = dev_states.light.get(update.dev_id).unwrap(); //FIXME
                        for (_, state) in map {
                            light_states.push(state);
                        }
                        let avg_light_state = LightState::avg(light_states);

                        //apply
                        let ui_state = element_states.get_mut(*eid).unwrap(); //FIXME
                        *ui_state = Some(SubdeviceState::Light(avg_light_state.clone()));
                    }

                    //zone
                    if let Some(zids) = dep_map.light.zone.get(&update.dev_id) {
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
                            let ui_state = element_states.get_mut(*eid).unwrap(); //FIXME
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
                    let ui_state = element_states.get_mut(dep_map.light.all).unwrap(); //FIXME
                    *ui_state = Some(SubdeviceState::Light(avg_light_state.clone()));
                },
                SubdeviceState::Switch(_) => todo!(),
            }
            ui_update_tx.send(element_states.clone()).await.unwrap(); //FIXME
        }
    }
}



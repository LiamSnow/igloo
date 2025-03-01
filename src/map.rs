use std::{collections::HashMap, error::Error, iter, sync::Arc};

use serde::Serialize;
use tokio::sync::{mpsc, Mutex};

use crate::{cli::model::SwitchState, command::{AveragedSubdeviceState, LightState, SubdeviceState, SubdeviceType, TargetedSubdeviceCommand}, config::{ElementValue, IglooConfig, UIElementConfig}, effects::EffectsState, providers::{esphome, DeviceConfig}, selector::Selection};

#[derive(Default)]
pub struct IDLut {
    /// zid (zone ID) -> start_did, end_did
    pub did_range: Vec<(usize, usize)>,
    /// zid, dev_name -> did (device ID)
    pub did: Vec<HashMap<String, usize>>,
    /// zone_name -> zid
    pub zid: HashMap<String, usize>,
}

pub struct IglooStack {
    /// did -> dev command channnel
    pub dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>>,
    pub lut: IDLut,
    pub elements: HashMap<String, Vec<Element>>,
    pub element_states: ElementStatesLock,
    pub element_values: Mutex<Vec<ElementValue>>,
    pub evid_lut: HashMap<String, usize>,
    pub effects_state: Mutex<EffectsState>,
    //permissions zid,uid,allowed
    // pub zone_perms: Vec<Vec<bool>>,
}

pub type ElementStatesLock = Arc<Mutex<Vec<Option<AveragedSubdeviceState>>>>;

#[derive(Serialize)]
pub struct Element {
    pub cfg: UIElementConfig,
    // element state index (for elements tied to devices IE Lights)
    pub esid: Option<usize>,
    // element value index (for elements that hold a value IE TimeSelector)
    pub evid: Option<usize>
}

#[derive(Clone)]
pub struct SubdeviceStateUpdate {
    pub did: usize,
    pub sid: usize,
    pub value: SubdeviceState
}

impl IglooStack {
    pub async fn init(config: IglooConfig) -> Result<Arc<Self>, Box<dyn Error>> {
        // Make IDs
        let (mut next_did, mut next_zid) = (0, 0);
        let mut lut = IDLut { ..Default::default() };
        let (mut dev_cfgs, mut dev_sels, mut on_update_of_types, mut on_subdev_updates)
            = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (zone_name, devs) in config.zones {
            let start_did = next_did;
            let mut did_lut = HashMap::new();
            for (dev_name, dev_cfg) in devs {
                did_lut.insert(dev_name.clone(), next_did);
                dev_cfgs.push(dev_cfg);
                on_update_of_types.push(None);
                on_subdev_updates.push(None);
                dev_sels.push(format!("{zone_name}.{dev_name}"));
                next_did += 1;
            }
            lut.did.push(did_lut);
            lut.did_range.push((start_did, next_did-1));
            lut.zid.insert(zone_name, next_zid);
            next_zid += 1;
        }

        // Make UI Elements
        let (mut elements, mut evid_lut, mut element_states, mut element_values) =
            (HashMap::new(), HashMap::new(), Vec::new(), Vec::new());
        let (mut esid, mut evid) = (0, 0);
        for (group_name, cfgs) in config.ui {
            let mut group_elements = Vec::new();
            for cfg in cfgs {
                if cfg.get_sel_and_subdev_type().is_some() {
                    let el = Element { cfg, esid: Some(esid), evid: None };
                    group_elements.push(el);
                    element_states.push(None);
                    esid += 1;
                }
                else if let Some(default_value) = cfg.get_default_value() {
                    evid_lut.insert(format!("{group_name}.{}", cfg.get_name().unwrap()), evid);
                    group_elements.push(Element { cfg, esid: None, evid: Some(evid) });
                    element_values.push(default_value);
                    evid += 1;
                }
                else {
                    group_elements.push(Element { cfg, esid: None, evid: None });
                }
            }
            elements.insert(group_name, group_elements);
        }

        // Make element state trackers
        let element_states = Arc::new(Mutex::new(element_states));
        for el in elements.values().flat_map(|group| group.iter()).filter(|el| el.esid.is_some()) {
            let esid = el.esid.unwrap();
            let (sel_str, subdev_type) = el.cfg.get_sel_and_subdev_type().unwrap();
            let sel = Selection::from_str(&lut, sel_str)?;

            if let Selection::Subdevice(_, did, subdev_name) = sel {
                if on_subdev_updates[did].is_none() {
                    on_subdev_updates[did] = Some(HashMap::new());
                }
                let (update_tx, update_rx) = mpsc::channel::<SubdeviceState>(5);
                on_subdev_updates[did].as_mut().unwrap().insert(subdev_name.clone(), update_tx);
                tokio::spawn(element_subdev_state_task(element_states.clone(), esid, subdev_type, update_rx));
            }
            else {
                let (start_did, end_did) = match sel {
                    Selection::All => (0, next_did-1),
                    Selection::Zone(_, start_did, end_did) => (start_did, end_did),
                    Selection::Device(_, did) => (did, did),
                    _ => panic!()
                };
                let (update_tx, update_rx) = mpsc::channel::<SubdeviceStateUpdate>(end_did-start_did+1); //TODO does this seem reasonable?
                for did in start_did..=end_did {
                    if on_update_of_types[did].is_none() {
                        on_update_of_types[did] = Some(HashMap::new());
                    }
                    let v = on_update_of_types[did].as_mut().unwrap().entry(subdev_type.clone()).or_insert(Vec::new());
                    v.push(update_tx.clone());
                }
                tokio::spawn(element_state_task(element_states.clone(), esid, subdev_type, start_did, end_did, update_rx));
            }
        }

        // Make Devices
        let mut dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>> = Vec::new();
        for did in 0..next_did {
            let dev_sel = dev_sels.remove(0);
            let dev_cfg = dev_cfgs.remove(0);
            let on_update_of_type = on_update_of_types.remove(0);
            let on_subdev_update = on_subdev_updates.remove(0);
            let (cmd_tx, cmd_rx) = mpsc::channel::<TargetedSubdeviceCommand>(5);

            let task = match dev_cfg {
                DeviceConfig::ESPHome(cfg) => esphome::task(cfg, did, dev_sel, cmd_rx, on_update_of_type, on_subdev_update),
                DeviceConfig::HomeKit(_cfg) => todo!(),
            };
            tokio::spawn(task);

            dev_chans.push(cmd_tx);
        }

        let element_values = Mutex::new(element_values);
        let effects_state = Mutex::new(EffectsState { next_id: 0, current: HashMap::new() });
        Ok(Arc::new(IglooStack { dev_chans, lut, elements, element_states, element_values, evid_lut, effects_state }))
    }
}

async fn element_state_task(element_states: ElementStatesLock, esid: usize, subdev_type: SubdeviceType, start_did: usize, end_did: usize, mut update_rx: mpsc::Receiver<SubdeviceStateUpdate>) {
    let num_devices = end_did-start_did+1;
    match subdev_type {
        SubdeviceType::Light => {
            let mut states: Vec<Vec<Option<LightState>>> = vec![Vec::new(); num_devices];

            while let Some(update) = update_rx.recv().await {
                let dev_states = states.get_mut(update.did - start_did).unwrap(); //FIXME

                //fill subdevice vec (only happens a few times)
                let needed_len = update.sid + 1;
                if dev_states.len() < needed_len {
                    dev_states.extend(iter::repeat(None).take(needed_len - dev_states.len()));
                }

                //update state
                if let SubdeviceState::Light(value) = update.value {
                    *dev_states.get_mut(update.sid).unwrap() = Some(value);
                }
                else {
                    panic!("Light element recieved wrong subdevice state update {:#?}", update.value.get_type())
                }

                //update master
                let (avg_state, homogeneous) = LightState::avg(&states).unwrap();
                let mut element_states = element_states.lock().await;
                element_states[esid] = Some(AveragedSubdeviceState { value: SubdeviceState::Light(avg_state), homogeneous });
            }
        },
        SubdeviceType::Switch => {
            let mut states: Vec<Vec<Option<bool>>> = vec![Vec::new(); num_devices];

            while let Some(update) = update_rx.recv().await {
                let dev_states = states.get_mut(update.did - start_did).unwrap(); //FIXME

                //fill subdevice vec (only happens a few times)
                let needed_len = update.sid + 1;
                if dev_states.len() < needed_len {
                    dev_states.extend(iter::repeat(None).take(needed_len - dev_states.len()));
                }

                //update state
                if let SubdeviceState::Switch(value) = update.value {
                    *dev_states.get_mut(update.sid).unwrap() = Some(value.into());
                }
                else {
                    panic!("Switch element recieved wrong subdevice state update {:#?}", update.value.get_type())
                }

                //update master
                let (avg_state, homogeneous) = SwitchState::avg(&states).unwrap();
                let mut element_states = element_states.lock().await;
                element_states[esid] = Some(AveragedSubdeviceState { value: SubdeviceState::Switch(avg_state), homogeneous });
            }
        }
    }

}

async fn element_subdev_state_task(element_states: ElementStatesLock, esid: usize, subdev_type: SubdeviceType, mut update_rx: mpsc::Receiver<SubdeviceState>) {
    while let Some(value) = update_rx.recv().await {
        let typ = value.get_type();
        if typ != subdev_type {
            println!("ERROR element type {:#?} does match subdev type {:#?}", subdev_type, typ);
        }

        let mut element_states = element_states.lock().await;
        element_states[esid] = Some(AveragedSubdeviceState { value, homogeneous: true });
    }
}

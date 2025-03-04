use std::{collections::HashMap, error::Error, iter, sync::Arc};

use axum::extract::ws::Utf8Bytes;
use bitvec::prelude::bitvec;
use bitvec::vec::BitVec;
use chrono::NaiveTime;
use serde::{Serialize, Serializer};
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::{
    auth::Auth, cli::model::{Cli, SwitchState}, command::{LightState, SubdeviceState, SubdeviceType}, config::UIElementConfig, stack::DeviceIDLut, permissions::Permissions, selector::Selection
};

pub struct Elements {
    /// [(group name, element in group)]
    pub elements: Vec<(String, Vec<Element>)>,
    /// esid -> Option<state>
    pub states: ElementStatesLock,
    /// evid -> Option<state>
    pub values: Mutex<Vec<ElementValue>>,
    pub evid_lut: HashMap<String, usize>,
}

pub type ElementStatesLock = Arc<Mutex<Vec<Option<AveragedSubdeviceState>>>>;

#[derive(Serialize, Clone)]
pub struct Element {
    pub cfg: UIElementConfig,
    // element state index (for elements tied to devices IE Lights)
    pub esid: Option<usize>,
    // element value index (for elements that hold a value IE TimeSelector)
    pub evid: Option<usize>,
    /// if None, anyone can see
    #[serde(skip_serializing)]
    pub allowed_uids: Option<BitVec>
}

#[derive(Clone)]
pub struct SubdeviceStateUpdate {
    pub did: usize,
    pub sid: usize,
    pub value: SubdeviceState,
}

#[derive(Serialize)]
struct ElementStateUpdate {
    esid: usize,
    value: AveragedSubdeviceState,
}

#[derive(Serialize, Clone)]
pub struct AveragedSubdeviceState {
    pub value: SubdeviceState,
    pub homogeneous: bool,
}

#[derive(Serialize)]
pub struct ElementValueUpdate {
    pub evid: usize,
    pub value: ElementValue,
}

#[derive(Debug, Serialize, Clone)]
pub enum ElementValue {
    #[serde(serialize_with = "serialize_time")]
    Time(NaiveTime),
}

pub fn serialize_time<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&time.format("%H:%M").to_string())
}

pub fn parse_time(time_str: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(&time_str, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(&time_str, "%I:%M %p"))
}

pub struct DeviceUpdateSubscribers {
    // maps did,type -> Subscribers
    pub of_types: Vec<Option<HashMap<SubdeviceType, Vec<mpsc::Sender<SubdeviceStateUpdate>>>>>,
    // maps did,subdev_name -> Subscribers
    pub subdev: Vec<Option<HashMap<String, mpsc::Sender<SubdeviceState>>>>,
}

impl Elements {
    pub fn init(
        ui: Vec<(String, Vec<UIElementConfig>)>,
        lut: &DeviceIDLut,
        ws_broadcast: broadcast::Sender<Utf8Bytes>,
        perms: &Permissions,
        auth: &Auth,
    ) -> Result<(Self, DeviceUpdateSubscribers), Box<dyn Error>> {
        // Fill subscribers
        let mut subscribers = DeviceUpdateSubscribers {
            of_types: vec![None; lut.num_devs],
            subdev: vec![None; lut.num_devs],
        };

        // Make UI Elements
        let (mut elements, mut evid_lut, mut states, mut values) =
            (Vec::new(), HashMap::new(), Vec::new(), Vec::new());
        let (mut esid, mut evid) = (0, 0);
        for (group_name, cfgs) in ui {
            let mut group_elements = Vec::new();
            for cfg in cfgs {
                let (mut el_esid, mut el_evid, mut allowed_uids) = (None, None, None);

                if let Some((sel, _)) = cfg.get_sel_and_subdev_type() {
                    allowed_uids = Selection::from_str(lut, sel)?.get_zid().and_then(|zid| perms.zone.get(zid)).cloned();
                    el_esid = Some(esid);
                    states.push(None);
                    esid += 1;
                } else if let Some(val) = cfg.get_def_val() {
                    evid_lut.insert(format!("{group_name}.{}", cfg.get_name().unwrap()), evid);
                    el_evid = Some(evid);
                    values.push(val);
                    evid += 1;
                }

                // if this calls commands, we have to parse them to see what they interact with for
                // permissions
                if let Some(cmd_strs) = cfg.get_commands() {
                    let all = bitvec![1; auth.num_users];
                    let mut uids = all.clone();
                    for cmd_str in cmd_strs {
                        let cmd = match Cli::parse(&cmd_str) {
                            Ok(r) => r,
                            Err(e) => return Err(format!("Element had invalid command: {}", e.render().to_string()).into())
                        };
                        if let Some(sel_str) = cmd.command.get_selection() {
                            let new_uids = match Selection::from_str(lut, sel_str)?.get_zid() {
                                Some(zid) => perms.zone.get(zid).unwrap().clone(),
                                None => all.clone(),
                            };
                            uids &= new_uids;
                        }
                    }
                    allowed_uids = Some(uids);
                }

                //TODO script perms

                group_elements.push(Element { cfg, esid: el_esid, evid: el_evid, allowed_uids });
            }
            elements.push((group_name, group_elements));
        }

        // Make element state trackers
        let states = Arc::new(Mutex::new(states));
        for el in elements
            .iter()
            .flat_map(|(_, els)| els.iter())
            .filter(|el| el.esid.is_some())
        {
            let esid = el.esid.unwrap();
            let (sel_str, subdev_type) = el.cfg.get_sel_and_subdev_type().unwrap();
            let sel = Selection::from_str(&lut, sel_str)?;

            if let Selection::Subdevice(_, did, subdev_name) = sel {
                if subscribers.subdev[did].is_none() {
                    subscribers.subdev[did] = Some(HashMap::new());
                }
                let (update_tx, update_rx) = mpsc::channel::<SubdeviceState>(5);
                subscribers.subdev[did]
                    .as_mut()
                    .unwrap()
                    .insert(subdev_name.clone(), update_tx);
                tokio::spawn(element_subdev_state_task(
                    states.clone(),
                    esid,
                    subdev_type,
                    update_rx,
                    ws_broadcast.clone(),
                ));
            } else {
                let (start_did, end_did) = match sel {
                    Selection::All => (0, lut.did.len() - 1),
                    Selection::Zone(_, start_did, end_did) => (start_did, end_did),
                    Selection::Device(_, did) => (did, did),
                    _ => panic!(),
                };
                let (update_tx, update_rx) =
                    mpsc::channel::<SubdeviceStateUpdate>(end_did - start_did + 1); //TODO does this seem reasonable?
                for did in start_did..=end_did {
                    if subscribers.of_types[did].is_none() {
                        subscribers.of_types[did] = Some(HashMap::new());
                    }
                    let v = subscribers.of_types[did]
                        .as_mut()
                        .unwrap()
                        .entry(subdev_type.clone())
                        .or_insert(Vec::new());
                    v.push(update_tx.clone());
                }
                tokio::spawn(element_state_task(
                    states.clone(),
                    esid,
                    subdev_type,
                    start_did,
                    end_did,
                    update_rx,
                    ws_broadcast.clone(),
                ));
            }
        }

        Ok((
            Self {
                elements,
                states,
                values: Mutex::new(values),
                evid_lut,
            },
            subscribers,
        ))
    }
}

async fn element_state_task(
    element_states: ElementStatesLock,
    esid: usize,
    subdev_type: SubdeviceType,
    start_did: usize,
    end_did: usize,
    mut update_rx: mpsc::Receiver<SubdeviceStateUpdate>,
    ws_broadcast: broadcast::Sender<Utf8Bytes>,
) {
    let num_devices = end_did - start_did + 1;
    match subdev_type {
        SubdeviceType::Light => {
            // did, sid => state
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
                } else {
                    panic!(
                        "Light element recieved wrong subdevice state update {:#?}",
                        update.value.get_type()
                    )
                }

                //averge state
                let avg_state = LightState::avg(&states).unwrap(); //FIXME

                //broadcast to websockets
                let wsu = ElementStateUpdate {
                    esid,
                    value: avg_state.clone(),
                };
                let json = serde_json::to_string(&wsu).unwrap(); //FIXME
                let _ = ws_broadcast.send(json.into()); //ignore error (nobody is listening right now)

                //update master
                let mut element_states = element_states.lock().await;
                element_states[esid] = Some(avg_state);
            }
        }
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
                } else {
                    panic!(
                        "Switch element recieved wrong subdevice state update {:#?}",
                        update.value.get_type()
                    )
                }

                //averge state
                let avg_state = SwitchState::avg(&states).unwrap(); //FIXME

                //broadcast to websockets
                let wsu = ElementStateUpdate {
                    esid,
                    value: avg_state.clone(),
                };
                let json = serde_json::to_string(&wsu).unwrap(); //FIXME
                let _ = ws_broadcast.send(json.into()); //ignore error (nobody is listening right now)

                //update master
                let mut element_states = element_states.lock().await;
                element_states[esid] = Some(avg_state);
            }
        }
    }
}

async fn element_subdev_state_task(
    element_states: ElementStatesLock,
    esid: usize,
    subdev_type: SubdeviceType,
    mut update_rx: mpsc::Receiver<SubdeviceState>,
    ws_broadcast: broadcast::Sender<Utf8Bytes>,
) {
    while let Some(value) = update_rx.recv().await {
        let typ = value.get_type();
        if typ != subdev_type {
            println!(
                "ERROR element type {:#?} does match subdev type {:#?}",
                subdev_type, typ
            );
        }

        //broadcast to websockets
        let wsu = ElementStateUpdate {
            esid,
            value: AveragedSubdeviceState {
                value: value.clone(),
                homogeneous: true,
            },
        };
        let json = serde_json::to_string(&wsu).unwrap(); //FIXME
        let _ = ws_broadcast.send(json.into()); //ignore error (nobody is listening right now)

        //update master
        let mut element_states = element_states.lock().await;
        element_states[esid] = Some(AveragedSubdeviceState {
            value,
            homogeneous: true,
        });
    }
}

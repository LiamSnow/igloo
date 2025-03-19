use std::{collections::HashMap, error::Error, sync::Arc};

use axum::extract::ws::Utf8Bytes;
use bitvec::vec::BitVec;
use serde::Serialize;
use tokio::sync::{broadcast, Mutex};

use crate::{
    auth::Auth,
    cli::model::Cli,
    config::UIElementConfig,
    device::DeviceIDLut,
    selector::Selection,
    subdevice::{SubdeviceState, SubdeviceType},
};

pub struct Elements {
    /// [(group name, element in group)]
    pub elements: Vec<(String, Vec<Element>)>,

    /// esid -> Option<state>
    pub states: ElementStatesLock,

    /// esid -> start_did,end_did,subdev_type
    esid_meta: Vec<(usize, usize, SubdeviceType)>,

    /// did -> subscription
    subscriptions: Vec<DeviceChangeSubscriptions>,

    /// broadcast JSON of changed element state
    pub on_change: broadcast::Sender<Utf8Bytes>,
}

pub type ElementStatesLock = Mutex<Vec<Option<AveragedSubdeviceState>>>;

#[derive(Serialize, Clone)]
pub struct Element {
    pub cfg: UIElementConfig,
    // element state index (for elements tied to devices IE Lights)
    pub esid: Option<usize>,
    /// if None, anyone can see
    #[serde(skip_serializing)]
    pub allowed_uids: Option<BitVec>,
}

#[derive(Clone)]
pub struct SubdeviceStateUpdate {
    pub did: usize,
    pub sid: usize,
    pub value: SubdeviceState,
}

#[derive(Serialize)]
struct ElementStateChange {
    esid: usize,
    value: Option<AveragedSubdeviceState>,
}

#[derive(Serialize, Clone)]
pub struct AveragedSubdeviceState {
    pub value: SubdeviceState,
    pub homogeneous: bool,
}

#[derive(Default, Clone)]
pub struct DeviceChangeSubscriptions {
    pub of_type: HashMap<SubdeviceType, Vec<usize>>,
    pub subdev: HashMap<String, usize>,
}

impl Elements {
    pub fn init(
        ui: Vec<(String, Vec<UIElementConfig>)>,
        lut: &DeviceIDLut,
        auth: &Auth,
    ) -> Result<Self, Box<dyn Error>> {
        let mut subscriptions = vec![DeviceChangeSubscriptions::default(); lut.num_devs];

        // Make UI Elements
        let (mut elements, mut states, mut did_ranges) = (Vec::new(), Vec::new(), Vec::new());
        let mut esid = 0;
        for (group_name, cfgs) in ui {
            let mut group_elements = Vec::new();
            for cfg in cfgs {
                let (mut el_esid, mut allowed_uids) = (None, None);

                if let Some((sel_str, subdev_type)) = cfg.get_meta() {
                    allowed_uids = Selection::from_str(lut, sel_str)?
                        .get_zid()
                        .and_then(|zid| auth.perms.get(zid))
                        .cloned();
                    el_esid = Some(esid);
                    states.push(None);

                    // add subscribers and did_ranges
                    let sel = Selection::from_str(&lut, sel_str)?;
                    if let Selection::Subdevice(_, did, subdev_name) = sel {
                        subscriptions[did].subdev.insert(subdev_name.clone(), esid);
                        did_ranges.push((did, did, subdev_type));
                    } else {
                        let (start_did, end_did) = match sel {
                            Selection::All => (0, lut.did.len() - 1),
                            Selection::Zone(_, start_did, end_did) => (start_did, end_did),
                            Selection::Device(_, did) => (did, did),
                            _ => panic!(),
                        };
                        for did in start_did..=end_did {
                            let v = subscriptions[did]
                                .of_type
                                .entry(subdev_type.clone())
                                .or_insert(Vec::new());
                            v.push(esid);
                        }
                        did_ranges.push((start_did, end_did, subdev_type));
                    }

                    esid += 1;
                }

                // parse commands to find required perms (IE for buttons)
                if let Some(cmd_str) = cfg.get_command() {
                    let cmd = match Cli::parse(&cmd_str) {
                        Ok(r) => r,
                        Err(e) => {
                            return Err(format!(
                                "Element had invalid command: {}",
                                e.render().to_string()
                            )
                            .into())
                        }
                    };
                    if let Some(sel_str) = cmd.command.get_selection() {
                        if let Some(zid) = Selection::from_str(lut, sel_str)?.get_zid() {
                            allowed_uids = Some(auth.perms.get(zid).unwrap().clone());
                        }
                    }
                    //TODO script perms
                }

                group_elements.push(Element {
                    cfg,
                    esid: el_esid,
                    allowed_uids,
                });
            }
            elements.push((group_name, group_elements));
        }

        Ok(Self {
            elements,
            states: Mutex::new(states),
            esid_meta: did_ranges,
            subscriptions,
            on_change: broadcast::Sender::new(20),
        })
    }
}

pub async fn on_device_update(
    dev_states: &Arc<Mutex<Vec<HashMap<String, SubdeviceState>>>>,
    elements: &Arc<Elements>,
    did: usize,
    subdev_name: &str,
    subdev_state: &SubdeviceState,
) {
    let subscriptions = &elements.subscriptions[did];
    let mut updates = Vec::new();

    // subdev elements
    for (_, esid) in subscriptions
        .subdev
        .iter()
        .filter(|(sn, _)| *sn == subdev_name)
    {
        let (_, _, expected_type) = &elements.esid_meta[*esid];
        let (state, update) = calc_element_state_subdev(*esid, expected_type, subdev_state.clone());
        updates.push(update);
        let mut states = elements.states.lock().await;
        states[*esid] = state;
    }

    // normal elements
    let subdev_type = subdev_state.get_type();
    if let Some(esids) = subscriptions.of_type.get(&subdev_type) {
        for esid in esids {
            let (start_did, end_did, expected_type) = &elements.esid_meta[*esid];
            let (state, update) =
                calc_element_state(dev_states, *esid, *start_did, *end_did, expected_type).await;
            updates.push(update);
            let mut states = elements.states.lock().await;
            states[*esid] = state;
        }
    }

    // broadcast updates
    let json = serde_json::to_string(&updates).unwrap(); //FIXME ?
    let _ = elements.on_change.send(json.into()); //ignore error (nobody is listening right now)
}

async fn calc_element_state(
    dev_states: &Mutex<Vec<HashMap<String, SubdeviceState>>>,
    esid: usize,
    start_did: usize,
    end_did: usize,
    expected_type: &SubdeviceType,
) -> (Option<AveragedSubdeviceState>, ElementStateChange) {
    let dev_states = dev_states.lock().await;
    let dev_states = &dev_states[start_did..=end_did];
    let vals: Vec<_> = dev_states.iter().flat_map(|h| h.values()).collect();
    let state = expected_type.avg(vals);
    (state.clone(), ElementStateChange { esid, value: state })
}

fn calc_element_state_subdev(
    esid: usize,
    expected_type: &SubdeviceType,
    subdev_state: SubdeviceState,
) -> (Option<AveragedSubdeviceState>, ElementStateChange) {
    let typ = subdev_state.get_type();
    if typ != *expected_type {
        println!(
            "ERROR element type {:#?} does match subdev type {:#?}",
            expected_type, typ
        );
    }

    let state = Some(AveragedSubdeviceState {
        value: subdev_state,
        homogeneous: true,
    });

    (state.clone(), ElementStateChange { esid, value: state })
}

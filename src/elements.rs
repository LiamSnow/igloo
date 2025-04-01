use std::{collections::HashMap, error::Error, sync::Arc};

use axum::extract::ws::Utf8Bytes;
use bitvec::vec::BitVec;
use serde::Serialize;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, span, Level};

use crate::{
    auth::Auth,
    cli::model::Cli,
    config::UIElementConfig,
    device::DeviceIDLut,
    entity::{AveragedEntityState, EntityState, EntityType},
    selector::Selection,
};

pub struct Elements {
    /// [(group name, element in group)]
    pub elements: Vec<(String, Vec<Element>)>,

    /// esid -> Option<state>
    pub states: ElementStatesLock,

    /// esid -> start_did,end_did,entity_type
    esid_meta: Vec<(usize, usize, EntityType)>,

    /// did -> subscription
    subscriptions: Vec<DeviceChangeSubscriptions>,

    /// broadcast JSON of changed element state
    pub on_change: broadcast::Sender<Utf8Bytes>,
}

pub type ElementStatesLock = Mutex<Vec<Option<AveragedEntityState>>>;

#[derive(Serialize, Clone)]
pub struct Element {
    pub cfg: UIElementConfig,
    /// element state ID (for elements tied to devices IE Lights)
    pub esid: Option<usize>,
    /// script ID
    pub sid: Option<u32>,
    /// if None, anyone can see
    #[serde(skip_serializing)]
    pub allowed_uids: Option<BitVec>,
}

#[derive(Serialize)]
struct ElementStateChange {
    esid: usize,
    value: Option<AveragedEntityState>,
}

#[derive(Default, Clone)]
pub struct DeviceChangeSubscriptions {
    pub of_type: HashMap<EntityType, Vec<usize>>,
    pub entity: HashMap<String, usize>,
}

impl Elements {
    pub fn init(
        ui: Vec<(String, Vec<UIElementConfig>)>,
        lut: &DeviceIDLut,
        auth: &Auth,
    ) -> Result<Self, Box<dyn Error>> {
        let span = span!(Level::INFO, "Elements");
        let _enter = span.enter();
        info!("initializing");

        let mut subscriptions = vec![DeviceChangeSubscriptions::default(); lut.num_devs];

        // Make UI Elements
        let (mut elements, mut states, mut did_ranges) = (Vec::new(), Vec::new(), Vec::new());
        let (mut next_esid, mut next_sid) = (0, 0);
        for (group_name, cfgs) in ui {
            let mut group_elements = Vec::new();
            for cfg in cfgs {
                let (mut esid, mut sid, mut allowed_uids) = (None, None, None);

                if let Some((sel_str, entity_type)) = cfg.get_meta() {
                    allowed_uids = Selection::from_str(lut, sel_str)?
                        .get_zid()
                        .and_then(|zid| auth.perms.get(zid))
                        .cloned();
                    esid = Some(next_esid);
                    states.push(None);

                    // add subscribers and did_ranges
                    let sel = Selection::from_str(&lut, sel_str)?;
                    if let Selection::Entity(_, did, entity_name) = sel {
                        subscriptions[did]
                            .entity
                            .insert(entity_name.clone(), next_esid);
                        did_ranges.push((did, did, entity_type));
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
                                .entry(entity_type.clone())
                                .or_insert(Vec::new());
                            v.push(next_esid);
                        }
                        did_ranges.push((start_did, end_did, entity_type));
                    }

                    next_esid += 1;
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

                // claim script ID
                if matches!(cfg, UIElementConfig::Script(..)) {
                    sid = Some(next_sid);
                    next_sid += 1;
                }

                group_elements.push(Element {
                    cfg,
                    esid,
                    sid,
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
    dev_states: &Arc<Mutex<Vec<HashMap<String, EntityState>>>>,
    elements: &Arc<Elements>,
    did: usize,
    entity_name: &str,
    entity_state: &EntityState,
) {
    let subscriptions = &elements.subscriptions[did];
    let mut updates = Vec::new();

    // single entity elements
    for (_, esid) in subscriptions
        .entity
        .iter()
        .filter(|(sn, _)| *sn == entity_name)
    {
        let (_, _, expected_type) = &elements.esid_meta[*esid];
        let (state, update) =
            calc_element_state_single_entity(*esid, expected_type, entity_state.clone());
        updates.push(update);
        let mut states = elements.states.lock().await;
        states[*esid] = state;
    }

    // normal elements
    let entity_type = entity_state.get_type();
    if let Some(esids) = subscriptions.of_type.get(&entity_type) {
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
    dev_states: &Mutex<Vec<HashMap<String, EntityState>>>,
    esid: usize,
    start_did: usize,
    end_did: usize,
    expected_type: &EntityType,
) -> (Option<AveragedEntityState>, ElementStateChange) {
    let dev_states = dev_states.lock().await;
    let dev_states = &dev_states[start_did..=end_did];
    let vals: Vec<_> = dev_states.iter().flat_map(|h| h.values()).collect();
    let state = expected_type.avg(vals);
    (state.clone(), ElementStateChange { esid, value: state })
}

fn calc_element_state_single_entity(
    esid: usize,
    expected_type: &EntityType,
    entity_state: EntityState,
) -> (Option<AveragedEntityState>, ElementStateChange) {
    let typ = entity_state.get_type();
    if typ != *expected_type {
        error!(
            "element type {:#?} does match entity type {:#?}",
            expected_type, typ
        );
    }

    let state = Some(AveragedEntityState {
        value: entity_state,
        homogeneous: true,
    });

    (state.clone(), ElementStateChange { esid, value: state })
}

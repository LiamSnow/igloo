
use std::sync::Arc;

use serde::Serialize;
use tracing::error;

use crate::{
    entity::{AveragedEntityState, EntityState, EntityType},
    state::IglooState,
};

use super::Elements;

#[derive(Serialize)]
struct ElementStateChange {
    esid: usize,
    value: Option<AveragedEntityState>,
}

#[derive(Serialize)]
struct WSChangeBody<'a, T: ?Sized + Serialize> {
    pub header: &'a str,
    pub body: &'a T,
}

pub fn broadcast_changes_to_ws<T: ?Sized + Serialize>(elements: &Elements, header: &str, body: &T) {
    let json = serde_json::to_string(&WSChangeBody { header, body }).unwrap(); //FIXME ?
    let _ = elements.on_change.send(json.into()); //ignore error (nobody is listening right now)
}

pub async fn on_device_update(
    istate: &Arc<IglooState>,
    did: usize,
    entity_name: &str,
    entity_state: &EntityState,
) {
    let observers = &istate.elements.observers[did];
    let mut updates = Vec::new();

    // single entity elements
    for (_, esid) in observers
        .entity
        .iter()
        .filter(|(sn, _)| *sn == entity_name)
    {
        let (_, _, expected_type) = &istate.elements.esid_meta[*esid];
        let (state, update) =
            calc_element_state_single_entity(*esid, expected_type, entity_state.clone());
        updates.push(update);
        let mut states = istate.elements.states.lock().await;
        states[*esid] = state;
    }

    // normal elements
    let entity_type = entity_state.get_type();
    if let Some(esids) = observers.of_type.get(&entity_type) {
        for esid in esids {
            let (start_did, end_did, expected_type) = &istate.elements.esid_meta[*esid];
            let (state, update) =
                calc_element_state(istate, *esid, *start_did, *end_did, expected_type).await;
            updates.push(update);
            let mut states = istate.elements.states.lock().await;
            states[*esid] = state;
        }
    }

    broadcast_changes_to_ws(&istate.elements, "states", &updates);
}

async fn calc_element_state(
    istate: &Arc<IglooState>,
    esid: usize,
    start_did: usize,
    end_did: usize,
    expected_type: &EntityType,
) -> (Option<AveragedEntityState>, ElementStateChange) {
    let dev_states = istate.devices.states.lock().await;
    let dev_states_sel = &dev_states[start_did..=end_did];

    // average states
    let vals: Vec<_> = dev_states_sel.iter().flat_map(|h| h.values()).collect();
    let mut state = expected_type.avg(vals);

    // find disconnected devices
    if let Some(state_in) = &mut state {
        let total = end_did-start_did+1;
        let mut disconnected: usize = 0;
        for a in dev_states_sel {
            if !a.get("connected").unwrap().unwrap_connection() {
                disconnected += 1;
            }
        }
        state_in.disconnection_stats = Some((disconnected, total));
    }

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
        disconnection_stats: None
    });

    (state.clone(), ElementStateChange { esid, value: state })
}

pub mod error;
pub mod state;
pub mod element;

use std::collections::HashMap;

use axum::extract::ws::Utf8Bytes;
use element::Element;
use error::ElementsInitError;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, span, Level};

use crate::{
    auth::Auth,
    config::{ScriptConfig, UIElementConfig},
    device::ids::DeviceIDLut,
    entity::{AveragedEntityState, EntityType},
};

pub struct Elements {
    /// [(group name, element in group)]
    pub elements: Vec<(String, Vec<Element>)>,
    /// esid -> Option<state>
    pub states: ElementStatesLock,
    /// esid -> start_did,end_did,entity_type
    esid_meta: Vec<(usize, usize, EntityType)>,
    /// did -> observers
    observers: Vec<DeviceStateObservers>,
    /// broadcast JSON of changed element state
    pub on_change: broadcast::Sender<Utf8Bytes>,
}

pub type ElementStatesLock = Mutex<Vec<Option<AveragedEntityState>>>;

#[derive(Default, Clone)]
pub struct DeviceStateObservers {
    /// entity_type -> esid's of observers
    pub of_type: HashMap<EntityType, Vec<usize>>,
    /// entity_name -> esid of observer (exclusive)
    pub entity: HashMap<String, usize>,
}

pub(crate) struct InitContext<'a> {
    lut: &'a DeviceIDLut,
    auth: &'a Auth,
    script_configs: &'a HashMap<String, ScriptConfig>,
    watchers: Vec<DeviceStateObservers>,
    states: Vec<Option<AveragedEntityState>>,
    did_ranges: Vec<(usize, usize, EntityType)>,
    next_esid: usize,
    next_script_id: u32,
}

impl Elements {
    pub fn init(
        ui: Vec<(String, Vec<UIElementConfig>)>,
        lut: &DeviceIDLut,
        auth: &Auth,
        script_configs: &HashMap<String, ScriptConfig>,
    ) -> Result<Self, ElementsInitError> {
        let span = span!(Level::INFO, "Elements");
        let _enter = span.enter();
        info!("initializing");

        let mut ctx = InitContext {
            lut,
            auth,
            script_configs,
            watchers: vec![DeviceStateObservers::default(); lut.num_devs],
            states: Vec::new(),
            did_ranges: Vec::new(),
            next_esid: 0,
            next_script_id: 0,
        };

        let mut elements = Vec::with_capacity(ui.len());
        for (group_name, cfgs) in ui {
            elements.push((
                group_name,
                cfgs.into_iter()
                    .map(|cfg| Element::new(cfg, &mut ctx))
                    .collect::<Result<Vec<_>, ElementsInitError>>()?,
            ));
        }

        Ok(Self {
            elements,
            states: Mutex::new(ctx.states),
            esid_meta: ctx.did_ranges,
            observers: ctx.watchers,
            on_change: broadcast::Sender::new(20), //FIXME size??
        })
    }
}


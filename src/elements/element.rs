use bitvec::vec::BitVec;
use serde::Serialize;

use crate::{
    cli::model::Cli, config::UIElementConfig, device::ids::DeviceIDSelection, scripts
};

use super::{error::ElementsInitError, InitContext};

#[derive(Serialize, Clone)]
pub struct Element {
    pub cfg: UIElementConfig,
    /// element state ID (for elements tied to devices IE Lights)
    pub esid: Option<usize>,
    /// script ID
    pub script_id: Option<u32>,
    /// if None, anyone can see
    #[serde(skip_serializing)]
    pub allowed_uids: Option<BitVec>,
}

impl Element {
    pub(crate) fn new(cfg: UIElementConfig, ctx: &mut InitContext) -> Result<Self, ElementsInitError> {
        Ok(match cfg {
            UIElementConfig::Button(..) => Self::make_button(cfg, ctx)?,
            UIElementConfig::Script(..) => Self::make_script(cfg, ctx)?,
            _ => Self::make_default(cfg, ctx)?,
        })
    }

    fn make_default(
        cfg: UIElementConfig,
        ctx: &mut InitContext,
    ) -> Result<Self, ElementsInitError> {
        let (sel_str, entity_type) = cfg.get_meta().unwrap();
        let sel = DeviceIDSelection::from_str(&ctx.lut, sel_str)?;

        let esid = ctx.next_esid;
        ctx.next_esid += 1;
        ctx.states.push(None);

        // add did ranges
        let (start_did, end_did) = sel.get_did_range(ctx.lut);
        ctx.did_ranges
            .push((start_did, end_did, entity_type.clone()));

        // get permissions
        let allowed_uids = sel
            .get_zid()
            .and_then(|zid| ctx.auth.perms.get(zid))
            .cloned();

        // add observers
        if let DeviceIDSelection::Entity(_, did, entity_name) = sel {
            ctx.watchers[did].entity.insert(entity_name.clone(), esid);
        } else {
            for did in start_did..=end_did {
                let v = ctx.watchers[did]
                    .of_type
                    .entry(entity_type.clone())
                    .or_insert(Vec::new());
                v.push(esid);
            }
        }

        Ok(Self {
            cfg,
            esid: Some(esid),
            script_id: None,
            allowed_uids,
        })
    }

    fn make_button(cfg: UIElementConfig, ctx: &mut InitContext) -> Result<Self, ElementsInitError> {
        let (_, cmd_str) = cfg.unwrap_button();

        let cmd = match Cli::parse(&cmd_str) {
            Ok(r) => r,
            Err(e) => {
                return Err(ElementsInitError::InvalidButtonCommand(e.render().to_string()).into())
            }
        };

        let mut allowed_uids = None;
        if let Some(sel_str) = cmd.cmd.get_selection() {
            if let Some(zid) = DeviceIDSelection::from_str(ctx.lut, sel_str)?.get_zid() {
                allowed_uids = Some(ctx.auth.perms.get(zid).unwrap().clone());
            }
        }

        Ok(Self {
            cfg,
            esid: None,
            script_id: None,
            allowed_uids,
        })
    }

    fn make_script(cfg: UIElementConfig, ctx: &mut InitContext) -> Result<Self, ElementsInitError> {
        let args = cfg.unwrap_script();

        //claim ID
        let script_id = Some(ctx.next_script_id);
        ctx.next_script_id += 1;

        let mut args: Vec<String> = args.split_whitespace().map(str::to_string).collect();
        let script_name = args.remove(0);

        Ok(Self {
            cfg,
            esid: None,
            script_id,
            allowed_uids: Some(scripts::calc_perms(
                ctx.lut,
                ctx.auth,
                ctx.script_configs,
                script_name,
                args,
            )?),
        })
    }
}

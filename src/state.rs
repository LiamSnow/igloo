use std::{error::Error, sync::Arc};

use crate::{
    auth::Auth,
    config::IglooConfig,
    device::{ids::DeviceIDLut, Devices},
    elements::Elements,
    scripts::{self, Scripts},
};

pub struct IglooState {
    pub devices: Devices,
    pub elements: Elements,
    pub auth: Auth,
    pub scripts: Scripts,
}

impl IglooState {
    pub async fn init(icfg: IglooConfig) -> Result<Arc<Self>, Box<dyn Error>> {
        let (dev_lut, dev_cfgs, dev_sels) = DeviceIDLut::init(icfg.devices);
        let auth = Auth::init(icfg.auth, &dev_lut).await?;
        let elements = Elements::init(icfg.ui, &dev_lut, &auth, &icfg.scripts)?;

        let (devices, state_txs) = Devices::init(dev_lut, dev_cfgs, dev_sels);

        let res = Arc::new(IglooState {
            elements,
            auth,
            devices,
            scripts: Scripts::init(icfg.scripts),
        });

        for state_tx in state_txs {
            state_tx.send(res.clone())
                .unwrap_or_else(|_| panic!("IglooState: Could not send state to Devices"));
        }

        scripts::spawn_boot(&res).await?;

        Ok(res)
    }
}

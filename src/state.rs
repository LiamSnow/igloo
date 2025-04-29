use std::{error::Error, sync::Arc};

use tokio::sync::oneshot;

use crate::{
    auth::Auth,
    config::IglooConfig,
    device::{DeviceIDLut, Devices},
    elements::Elements,
    scripts::{self, Scripts},
};

pub struct IglooState {
    pub devices: Devices,
    pub elements: Arc<Elements>,
    pub auth: Auth,
    pub scripts: Scripts
}

impl IglooState {
    pub async fn init(icfg: IglooConfig) -> Result<Arc<Self>, Box<dyn Error>> {
        let (dev_lut, dev_cfgs, dev_sels) = DeviceIDLut::init(icfg.devices);

        let auth = Auth::init(icfg.auth, &dev_lut).await?;

        let elements = Arc::new(Elements::init(icfg.ui, &dev_lut, &auth, &icfg.scripts)?);

        let (state_tx, state_rx) = oneshot::channel();
        let devices = Devices::init(dev_lut, dev_cfgs, dev_sels, elements.clone(), state_rx);

        let scripts = Scripts::init(icfg.scripts);

        let res = Arc::new(IglooState {
            devices,
            elements,
            auth,
            scripts
        });

        state_tx.send(res.clone())
            .unwrap_or_else(|_| panic!("IglooState: Could not send state to Devices"));

        scripts::spawn_boot(&res).await?;

        Ok(res)
    }
}

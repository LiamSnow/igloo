use std::{error::Error, sync::Arc};

use crate::{
    auth::Auth,
    config::IglooConfig,
    device::{DeviceIDLut, Devices},
    elements::Elements,
    scripts::Scripts,
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

        let auth = Auth::init(icfg.auth, &dev_lut)?;

        let elements = Arc::new(Elements::init(icfg.ui, &dev_lut, &auth)?);

        let devices = Devices::init(dev_lut, dev_cfgs, dev_sels, elements.clone());

        let scripts = Scripts::init(icfg.scripts);

        Ok(Arc::new(IglooState {
            devices,
            elements,
            auth,
            scripts
        }))
    }
}

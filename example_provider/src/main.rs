use std::collections::HashMap;

use igloo_interface::{
    Color, ComponentValue, Device, Dimmer, Entities, Entity, FloeCommand, FloeResponse,
    IglooCommand, Switch,
    floe::{FloeInterfaceManager, ProtocolResult},
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> ProtocolResult<()> {
    let manager = FloeInterfaceManager::new(|command| async move {
        match command {
            IglooCommand::Ping => FloeResponse::Ok(None),
            IglooCommand::Update(update) => FloeResponse::Ok(None),
            IglooCommand::Config(config) => FloeResponse::Ok(None),
            IglooCommand::Custom(name, data) => FloeResponse::Ok(Some(format!("Handled {}", name))),
        }
    });

    manager.log("Floe started".to_string()).await?;

    let device = Device {
        name: "Test Device".to_string(),
        entities: Entities(HashMap::from([(
            "RGBCT_Bulb".to_string(),
            Entity(vec![
                ComponentValue::Light,
                ComponentValue::Switch(Switch(true)),
                ComponentValue::Dimmer(Dimmer(255)),
                ComponentValue::Color(Color { r: 255, g: 0, b: 0 }),
            ]),
        )])),
    };

    for _ in 0..5 {
        let uuid = Uuid::now_v7();
        let response = manager
            .send_command(FloeCommand::AddDevice(uuid, device.clone()))
            .await?;
        if response.is_ok() {
            break;
        }
    }

    tokio::signal::ctrl_c().await?;

    Ok(())
}

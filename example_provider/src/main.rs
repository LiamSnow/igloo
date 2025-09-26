use std::collections::HashMap;

use igloo_interface::{
    Color, ComponentUpdate, Dimmer, Entities, Entity, InitPayload, Light, Switch,
    floe::{FloeHandler, IglooInterface, IglooInterfaceError},
};
use uuid::Uuid;

#[derive(Default)]
struct ExampleFloe {}

impl FloeHandler for ExampleFloe {
    async fn init(&mut self, init: InitPayload, manager: &IglooInterface) {
        manager.log(format!("got init: {init:#?}")).await.unwrap();

        let mut entity = Entity::default();
        entity.set_light(Light);
        entity.set_switch(Switch(true));
        entity.set_dimmer(Dimmer(0.5));
        entity.set_color(Color { r: 255, g: 0, b: 0 });

        let device_id = Uuid::now_v7();
        let device_name = "Test Device".to_string();
        let entities = Entities(HashMap::from([("RGBCT_Bulb".to_string(), entity)]));

        manager
            .add_device(device_id, device_name, entities)
            .await
            .unwrap();

        manager.save_config("test\n".to_string()).await.unwrap();
    }

    async fn updates_requested(&mut self, updates: Vec<ComponentUpdate>, manager: &IglooInterface) {
        manager
            .log(format!("got req update: {updates:#?}"))
            .await
            .unwrap();
    }

    async fn custom(&mut self, name: String, data: String, manager: &IglooInterface) {
        manager
            .log(format!("got custom: name={name}, data={data}"))
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), IglooInterfaceError> {
    let handler = ExampleFloe::default();
    IglooInterface::run(handler).await?;
    Ok(())
}

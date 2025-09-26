use std::fs;

use crate::{AlarmState, Entities, Entity, Int, Light, Unit};

#[test]
fn test_output() {
    let mut entity1 = Entity::default();
    entity1.set_light(Light);
    entity1.set_int(Int(10));
    entity1.set_unit(Unit::VoltAmpereReactive);
    entity1.set_alarm_state(AlarmState::Disarmed);

    let mut entity2 = Entity::default();
    entity2.set_unit(Unit::Custom("10".to_string()));
    entity2.set_alarm_state(AlarmState::Custom("Random".to_string()));

    let mut entities = Entities::default();
    entities.insert("entity1".to_string(), entity1);
    entities.insert("entity2".to_string(), entity2);

    let contents = serde_json::to_string_pretty(&entities).unwrap();

    fs::write("out.json", contents).unwrap();
}

# IGLOO ESPHOME

What do we need to do?

Read config, connect requested devices

Config format should have Igloo ID, Connection Props

Register custom function `add_device(ip: String, noise_psk: Option<String>, password: Option<String>)`
   - This should save to config, then connect that devices

Register connected devices to Igloo
 - Requires building in-memory Entity Key <-> Entity Name
 - Also requires making ESPHome Entity -> Igloo Entities

Handle component change requests
 - Requires mapping somehow



## Details

### `ListEntitiesRequest`
Returns a bunch of `ListEntitiesXXXResponse`

For example:

```ron
ListEntitiesLightResponse {
    object_id: "rgbct_bulb",
    key: 3868629491,
    name: "RGBCT_Bulb",
    unique_id: "athom-rgbct-light-b4b2fflightrgbct_bulb",
    supported_color_modes: [
        11,
        35,
    ],
    legacy_supports_brightness: true,
    legacy_supports_rgb: true,
    legacy_supports_white_value: false,
    legacy_supports_color_temperature: true,
    min_mireds: 153.0,
    max_mireds: 500.0,
    effects: [],
    disabled_by_default: false,
    icon: "",
    entity_category: None,
}
ListEntitiesSensorResponse {
    object_id: "uptime_sensor",
    key: 683514054,
    name: "Uptime Sensor",
    unique_id: "d8bc38b4b2ff-uptime",
    icon: "mdi:timer-outline",
    unit_of_measurement: "s",
    accuracy_decimals: 0,
    force_update: false,
    device_class: "duration",
    state_class: StateClassTotalIncreasing,
    last_reset_type: LastResetNone,
    disabled_by_default: false,
    entity_category: Diagnostic,
} 
```

What we need to do with this:
 1. Request entities:
   - Construct `key` -> `name` map for handling state updates
   - Construct `name` -> `key` map for sending updates
   - For Any -> Case-by-case `Icon`, `Diagnostic`, `Config`
   - For Light -> Add `Light` components
   - For Sensor -> Add `Unit`, `Icon`, `DeviceClass`, `AccuracyDecimals` components
   - 


### `SubscribeStatesRequest` 

Sends all states at first, then sends updates:

```ron
BinarySensorStateResponse {
    key: 939730931,
    state: true,
    missing_state: false,
}
LightStateResponse {
    key: 3868629491,
    state: true,
    brightness: 1.0,
    color_mode: 11,
    color_brightness: 1.0,
    red: 1.0,
    green: 0.5372549,
    blue: 0.0,
    white: 1.0,
    color_temperature: 500.0,
    cold_white: 1.0,
    warm_white: 1.0,
    effect: "",
} 
```


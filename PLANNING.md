# Igloo Planning Document

## Device Tree

Igloo runs on an ECS system similar to Bevy:
 - **Device**: Usually represents a single hardware device.
   - It is entirely managed by the provider (providers usually have multiple devices)
   - It has a name (ex. "Ceiling Light"), permissions, and a collection of Entities (each named)
 - **Entity**: A collection of components that is composed into multiple different things. For example, a Light Bulb can be thought of as specifically a Light Bulb, but also something that is Switchable, Dimmable, and Colorable
 - **Components**: Part of an entities. Some contain values and some are just markers

Then we can organize our home with **Zones** (ex. Kitchen) which are simly groupings of Devices.
Devices may be in multiple Zones.

```json5
{
  "ceiling_light": {
    "provider": "ESPHome",
    "perms": "Inherit",
    "entities": {
      "RGBCT_Bulb": [
        {
          "Light": null // just a marker, Light is defined as requiring Switch, optionally having Dimmable, Colorable
        },
        {
          "Switch": {
            "value": true
          }
        },
        {
          "Dimmer": {
            "value": 1.0
          }
        },
        {
          "Color": {
            "value": {
              "r": 255,
              "g": 0,
              "b": 0
            }
          }
        }
      ],
      "Status": [
        {
          "Bool": {
            "value": true
          }
        }
      ],
      "Safe Mode": [
        {
          "Bool": {
            "value": false
          }
        }
      ],
      "Uptime Sensor": [
        {
          "LongSensor": {
            "unit": "seconds",
            "value": 128231289
          }
        }
      ],
      "IP Address": [
        {
          "String": {
            "value": "192.168.1.201"
          }
        }
      ],
      "Mac Address": [
        {
          "String": {
            "value": "..."
          }
        }
      ],
      "Connected SSID": [
        {
          "String": {
            "value": "..."
          }
        }
      ],
      "Firmware": [
        {
          "String": {
            "value": "..."
          }
        }
      ]
    }
  }
}
```

Rust code:
```rust
#[derive(Component)]
#[require(Switch)]
#[optional(Dimmer)]
#[optional(Color)]
struct Light;

#[derive(Component)]
struct Switch {
  value: bool
}

#[derive(Component)]
struct Dimmer {
  value: f64
}

#[derive(Component)]
struct Color {
  r: u8,
  g: u8,
  b: u8,
}
```
  
## Floes

See [FLOES.md](FLOES.md)




## Server File Structure
```bash
igloo       # binary
auth.ron
state.ron
penguin.ron
penguin/
  SCRIPT.ron
  ...
dashboards/
  DASHBOARD.ron
  ...
floes/
  FLOE/
    BINARY
    Floe.toml
  ...
```



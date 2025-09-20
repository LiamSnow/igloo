# Igloo Planning Document

## Data

Was originally going to use SQLite, but lowkey overkill.
Simply store data inside Rust and persist back to ron files.


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


## Penguin
The node based visual programming made in Bevy!

## Hierarchy

Example Device Tree
```ron
{
  "kitchen": Zone(
    display_name: "Kitchen",
    devices: {
      "ceiling_light": Device(
        provider: String("ESPHome"),
        perms: Inherit,
        state: {
          // in this case Light and LongSensor are strongly typed structs, just like Color
          "RGBCT_Bulb": Light(
            on: true, // dont need Bool(true), because it takes a bool instead of a Component
            brightness: 1.0,
            color: Color(r: 255, g: 0, b: 0),
          ),
          "Status": Bool(true),
          "Safe Mode": Bool(false),
          "Uptime Sensor": LongSensor(
            unit: "seconds",
            value: 12310927398,            
          ),
          "IP Address": String("192.168.1.201"),
          "Mac Address": String("..."),
          "Connected SSID": String("..."),
          "Firmware": String("..."),
        }
      )
    }
  ),
}
```



### Components

Components, contain a `name`, a `type`, and a `state`.

**Primitive Types**:
 - `Float`: 64-bit floating point number
 - `Int`: 32-bit signed integer
 - `Long`: 128-bit signed integer
 - `String`: 
 - `ValidatedString(min_length, max_length, regex)` 
 - `Bool`: boolean
 - `Trigger`: no type, triggers a event
 - `Date`: date with day, month, year
 - `Time`: 24-hour time (hour, minute, second)
 - `Datetime`: `date` + `time`
 - `Duration`: TODO
 - `Color`: 24-bit RGB color
 - `Temperature`: `float` stored as celcius
 - `Uuid`: TODO
 - `Schedule`: TODO
 - `Url`: TODO
 - `Coordinate`: TODO
 - `Binary`: TODO

**Composite Types**:
 - `Array(T, N)`: fixed-length
 - `List(T)`: variable-length
 - `Tuple([T1, T2, ..])`: fixed tuples
 - `Optional(T)`: optional type
 - `{ field1: T1, field2: T2 }`: custom object
 - `Enum([V1, V2, ..])`: enumeration

**Custom Types**:
Custom types are defined in a registry on [igloo.io](igloo.io).
All types are backwards compatible. Conflicts should not come up
because if a new Floe comes out using a new `Light` feature,
the registry would have already been updated (registry gets
refreshed before any update/install of Floes).

For example:

```ron
// this Light type can represent basically any light
//  - A basic light (on/off only)
//  - An RGBCT light (on/off, brightness, CT, RGB)
//  - etc.
(
  name: Light,
  components: {
    "on": Bool,
    "brightness": Optional(Float),
    "color_temp": Optional(Float),
    "color": Optional(Color)
  }
)
```

```ron
(
  name: FloatSensor,
  components: {
    "icon": Optional(String),
    "unit": String,
    "value": Float
  }
)
```


### Devices
Devices are nothing more than a grouping of components.
They have a `name`, list of Components, `provider`, and `config` (used by provider)

For example, an Athom RGBCT Light could be represented as
(note its not actually stored in yaml, just for demonstration):

```yaml
name: kitchen_ceiling
components:
  RGBCT_Bulb: Light
  Status: bool
  Safe Mode: bool
  Uptime Sensor: Sensor # would publish state as unit: seconds, ..
  WiFi Signal: Sensor # unit dBm
  IP Address: string
  Mac Address: string
  Reset: trigger
  Connected SSID: string
  Firmware: string
```

See [FLOES.md](FLOES.md)





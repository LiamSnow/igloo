# Floes
Floes are extensions to Igloo.

> `floe` (`/flō/`)
> _noun_
> a sheet of floating ice.

They can do the following:
 - (Be a) Provider (IE ESPHome, Apple HomeKit):
   - Runs a program that commicates over std protocol via stdin/stdout to control devices
 - Add nodes to Penguin
 - Add Dashboard elements

# Floe Provider Protocol (JSON-based)

## Structure

**Command**:
```jsonl
{
  "command": string,
  "payload": Any  // optional, depends on command
}
```

**Ok Response**:
```jsonl
{
  "status": "ok",
  "payload": Any  // optional, depends on command
}
```

**Error Response**:
```jsonl
{
  "status": "error",
  "error": string
}
```

## Floe -> Igloo

**Commands**:

`"add_device"` registers a new device under this provider
 - payload: `DeviceInfo`
 - response: ok, payload = `{ "device": uuid }`

`"remove_device"`
 - payload: `{ "device": uuid }`
 - response: ok (no payload)

`"update_device"`
 - payload: `{ "device": uuid, "info": DeviceInfo }`
 - response: ok (no payload) | error (device doesn't exist, provider doesnt have perms, ..)

`"list_devices"` lists devices registered under this provider
 - payload: none
 - response: `{ uuid1: DeviceInfo, uuid2: DeviceInfo, .. }`

`"update_component"`
 - payload: `{ device: uuid, component: string, value:  }`
 - response: ok (no payload) | error

`""`

**Types**:

`uuid`: string UUID v7

`DeviceInfo`:
```jsonl
{
  "name": string,
  "components": {
    
  }
}
```

### Igloo -> Floe

Commands:
 - `"delete_device"` 
   - payload: JSON according to Floe's `Floe.ron` spec
   - response: ok (no payload) | error

### Example
Going to back to our Athom RGBCT Light example, lets walk through some examples.

```ron
Device(
  name: "kitchen_ceiling",
  components: {
    "RGBCT_Bulb": Light,
    "Status": Bool,
    "Safe Mode": Bool,
    "Uptime Sensor": Sensor, // would publish state as unit: seconds, ..
    "WiFi Signal": Sensor, // unit dBm
    "IP Address": String,
    "Mac Address": String,
    "Reset": Trigger,
    "Connected SSID": String,
    "Firmware": String,
  }
)
```

**ESPHome Example**:
 1. User adds ESPHome Floe
 2. Igloo installs Floe and reads it's `Floe.ron`
 3. Igloo spawns Floe's binary
 4. User clicks "Add Device" button and fills out modal (action from `Floe.ron`)
 5. Igloo sends `add_device` command to Floe
 6. 


Example `Floe.ron`:

```ron
(
  name: "ESPHome",
  version: "0.1.0",
  authors: ["Liam Snow"],
  license: "MIT",

  provider: 
)
```

```yaml
name: ESPHome,
provider:
  # `config` only defines the editable config in the menu
  # since esphome's config is edited by "add_device"
  # it has no config
  config:
  actions:
    add_device:
      title: Add Device
      parameters:
        name: string
        ip: string
        noise_psk: string?
        password: string?
```

**Apple HomeKit Example**:
 1. 

```yaml

```

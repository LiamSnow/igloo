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

## Device Tree

 - **Zone**: A grouping of Devices. Devices may be in multiple groups
 - **Device**: A specific hardware device. Contains a Component Tree.
    - Providers can product multiple devices and modify their components
    - Can represent multiple different things. For example, we might have one device that has 5 switches
 - **Component**: A typed, specific object. For example a switch, light bulb, etc.

Example Device Tree
```json
{
  "ceiling_light": {
    "provider": "ESPHome",
    "perms": "Inherit",
    "state": {
      "RGBCT_Bulb": {
        "type": "Light",
        "on": true,
        "brightness": 1.0,
        "color": {
          "r": 255,
          "g": 0,
          "b": 0
        }
      },
      "Status": true,
      "Safe Mode": false,
      "Uptime Sensor": {
        "type": "LongSensor",
        "unit": "seconds",
        "value": 12310927398
      },
      "IP Address": "192.168.1.201",
      "Mac Address": "...",
      "Connected SSID": "...",
      "Firmware": "..."
    }
  }
}
```


See [FLOES.md](FLOES.md)





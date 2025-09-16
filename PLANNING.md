# Igloo Planning Document

## Database

1 SQLite Database

### Tables
 - users: UID, username, password
 - groups: GID, name, \[UID\]
 - sessions: token, UID, expiration
 - floes: 

## Penguin
The node based visual programming made in Bevy!

## Hierarchy

```
Zone
 -> Device..
    -> Component..
```

### Devices




### Components
 - Float
 - UnsignedInteger
 - SignedInteger
 - Text
 - Boolean
 - Percent (0.0 - 1.0)
 - Temperature (Celcius but can be displayed as Fahrenheit)
 - ColorTemperature (Kelvin)


### Traits
Igloo leverages a composition style (instead of inheritance). TODO bruh bad wording

There is no `Light` components or devices in Igloo BUT there is a `Light` trait.
This functionality is preferred because it is very extensible and maintainable.

Traits are only allowed inside Igloo (IE Floes cannot create traits). This is done
so that there is no conflict of traits. Please make a PR if you need a new Trait.

---

For example, lets say we want to represent an Athom RGBCT Light Bulb.

The device contains the following components:
 - brightness (`Percent`)
 - red (`Percent`)
 - green (`Percent`)
 - blue (`Percent`)
 - color_temperature (`ColorTemperature`)
 - on (`Boolean`)
 - ...others

Then it would implement `RGBCTBLight(on, brightness, color_temperature, red, green, blue)`

Which means that it can be used in any function taking a `RGBCTBLight`, `RGBLight`, `CTLight`, `Light`...

For a light we have:
```
Light: on/off control
  - CTLight: + color temperature
  - RGBLight: + rgb control
  - BLight: + brightness
```









## Floes
Floes are extensions to Igloo.

> `floe` (`/flō/`)
> _noun_
> a sheet of floating ice.

They can do the following:
 - (Be a) Provider (IE ESPHome, Apple HomeKit):
   - Runs a program that commicates over std protocol via stdin/stdout to control devices
 - Add nodes to Penguin
 - Run code with no interface/side load (ex. MQTT broker)
 - Add UI elements

### Provider Protocol

Floe -> Igloo
 - Register new device(ID, name, ...)
 - Unregister device
 - Get register devices
 - Update device status(ID)


Igloo -> Flow
 - 

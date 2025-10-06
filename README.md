# igloo WIP

A secure, fast, & intuitive smart home platform.

## Penguin
A visual-programming language for the smart home world.

TODO

## Floes
Floes are extensions to Igloo.

> `floe` (`/flō/`)
> _noun_
> a sheet of floating ice.

They can do the following:
 - (Be a) Provider (IE ESPHome, Apple HomeKit):
   - Runs a program that commicates over std protocol via stdin/stdout to control devices
 - Add nodes to Penguin
 - Add Dashboard elements

## ECS
TODO fix this

Igloo runs on an ECS system similar to Bevy:
 - **Device**: Usually represents a single hardware device.
   - It is entirely managed by the provider (providers usually have multiple devices)
   - It has a name (ex. "Ceiling Light"), permissions, and a collection of Entities (each named)
 - **Entity**: A collection of components that is composed into multiple different things. For example, a Light Bulb can be thought of as specifically a Light Bulb, but also something that is Switchable, Dimmable, and Colorable
 - **Components**: Part of an entities. Some contain values and some are just markers

Then we can organize our home with **Zones** (ex. Kitchen) which are simply groupings of Devices.
Devices may be in multiple Zones.


## Architecture
All Rust 🦀
 - **Frontend**: Leptos + Bevy (for Penguin)
 - **Backend**: Axum


## Server File Structure
```bash
igloo       # binary
auth.toml
state.toml
penguin.toml
penguin/
  SCRIPT.toml
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

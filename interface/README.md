# Igloo Interface

Communications interface between Igloo and Floes.

TODO docs (for now see `../example_provider`)

## PROTOCOL SPEC
`CMD_ID, BYTES`
For example
 - `CMD_ID == 64` decode bytes as `Int` (`i32`)
 - `CMD_ID = 107` decode bytes as `Color`

## Component Generation
Components are generated from [components.toml](components.toml)

//!
//!
//!
//! # File structure
//!
//! ```txt
//! data/history/
//!   {ID}.bin* (primary file)
//!   {ID}_strings.bin* (optional, string intern table)
//! ```
//!
//! Where `{ID}` is the following joined by `_`:
//!  - DeviceID (u64) in BE bytes base58 (default display)
//!  - EntityID (String) UTF-8 as BE bytes in base58 (avoids bad filename issue)
//!    - EntityID is always capped to a len of 100, avoiding too long of filename
//!  - ComponentType (Enum) as String
//!
//! ## Primary File Format
//!  - [u16] major version
//!  - [u32] length of metadata
//!  - [HistoricalInstanceMetadata] metadata in postcard format
//!  - Entry*, where each entry is:
//!    - [u32] offset from `start_timestamp` in metadata
//!    - [_] value of component
//!
//! ## String Intern Table Format
//!
//!
//! # Versioning
//!  

pub const VERSION: u16 = 0;

struct HistoricalInstanceMetadata {
    /// Sanity check
    /// For primitive components, this will be the size of serialized data
    /// For string|enum components, this will be the size of the InternID
    entry_size_bytes: u16,

    ///
    start_timestamp: u64,
    max_age_hours: Option<u32>,
    min_interval_ms: u32,
}

## Changelog

### v0.12.1

* Restore `std` and remove `heapless` from default features 

### v0.12

* Bump Rust version to 1.86
* Bump `heapless` to 0.9.1
* Make `heapless` a default feature and remove `std` 

### v0.11

* Replace error code and function code consts with enums

### v0.10

* `ModbusContext` has been extended with methods to set/get boolean registers
  with `u8`-slices

### v0.9

* Added [defmt](https://crates.io/crates/defmt) support (via a feature)

* ModbusContext has become a trait

* Bug fixes and code refactoring

### v0.8.0

* Events system (experimental)

* ModbusFrameBuf is no longer mandatory. The crate functions accept frames of
  any length (u8-slices).

### v0.7.0

* ModbusContext has become a struct with custom context sizes with generic
  constants
* Use ModbusContextSmall for small contexts and ModbusContextFull for big ones
* Removed built-in dump/restore methods (use serde, bincode or custom ones)

### v0.6.3

* Moved modbus constants into a `consts` module.
* Reworked the crate dependency and cargo-feature structure. Features are now
  additive only. Instead of the `nostd` feature, a `std` one can be opted-out
  by specifying `default-features = false`. Similarly, instead of the
  `smallcontext` feature, there is now a `fullcontext` feature which is also
  enabled by default.
* Crate structure cleanup.

### v0.6

* `guess_request_frame_len` function now supports TCP (and perhaps UDP)
* huge code refactoring, fixed and formatted for the nowadays Rust standards
* majority of functions correctly check overflows and report errors instead of
  invalid values/panics

### v0.5

* Common functions and structures moved to main crate module
* Modbus client

### v0.4

* Modbus context is no longer created automatically and no mutex guard is
  provided by default. Use ModbusContext::new() to create context object and
  then use it as you wish - protect with any kind of Mutex, with RwLock or just
  put into UnsafeCell.
* Context SDK changes: all functions moved inside context, removed unnecessary
  ones, function args optimized.
* FixedVec support included by default, both in std and nostd.
* Added support for 64-bit integers

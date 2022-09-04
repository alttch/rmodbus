## Changelog

### v0.6.3

* Moved modbus constants into a `consts` module.
* Reworked the crate dependency and cargo-feature structure.  Features are now
  additive only.  Instead of the `nostd` feature, you now have to opt-out of
  the `std` feature by specifying `default-features = false`.  Similarly,
  instead of the `smallcontext` feature, there is now a `fullcontext` feature
  which is also enabled by default.
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

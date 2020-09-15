#![no_std]

include!("fake-mutex.rs");

macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock()
    };
}

include!("nostd-common.rs");

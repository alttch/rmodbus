#![no_std]
use spin::{Mutex, MutexGuard};

macro_rules! lock_mutex {
    ($v:path) => {
        $v.lock()
    };
}

include!("nostd-common.rs");

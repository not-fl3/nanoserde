//! # nanoserde

//! [![Github Actions](https://github.com/not-fl3/nanoserde/workflows/Cross-compile/badge.svg)](https://github.com/not-fl3/nanoserde/actions?query=workflow%3A)
//! [![Crates.io version](https://img.shields.io/crates/v/nanoserde.svg)](https://crates.io/crates/nanoserde)
//! [![Documentation](https://docs.rs/nanoserde/badge.svg)](https://docs.rs/nanoserde)
//! [![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/WfEp6ut)
//!
//! Data serialization library with zero dependencies. No more syn/proc_macro2/quote in the build tree!
//!
//! The main difference with "serde" and the reason why "nanoserde" is possible: there is no intermediate data model
//! For each serialisation datatype there is a special macro.
//!
//! Derive macros available: `DeJson`, `SerJson`, `DeBin`, `SerBin`, `DeRon`, `SerRon`
//!
//! `nanoserde` supports some serialization customisation with `#[nserde()]` attributes.
//! For `#[nserde(..)]` supported attributes for each format check [Features support matrix](https://github.com/not-fl3/nanoserde#features-support-matrix)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(any(feature = "binary", feature = "json", feature = "ron"))]
pub use nanoserde_derive::*;

#[cfg(feature = "binary")]
mod serde_bin;
#[cfg(feature = "binary")]
pub use crate::serde_bin::*;

#[cfg(feature = "ron")]
mod serde_ron;
#[cfg(feature = "ron")]
pub use crate::serde_ron::*;

#[cfg(feature = "json")]
mod serde_json;
#[cfg(feature = "json")]
pub use crate::serde_json::*;

#[cfg(feature = "toml")]
mod toml;
#[cfg(feature = "toml")]
pub use crate::toml::*;

#[cfg(test)]
mod format_test {
    use std::time::{Duration, SystemTime};
    
    #[test]
    fn check_current_format() {
        let dur = Duration::new(1000, 999_999_999);
        let time = SystemTime::UNIX_EPOCH + Duration::new(1633072800, 500_000_000);
        
        println!("JSON Duration: {}", crate::SerJson::serialize_json(&dur));
        println!("JSON SystemTime: {}", crate::SerJson::serialize_json(&time));
        
        #[cfg(feature = "ron")]
        {
            println!("RON Duration: {}", crate::SerRon::serialize_ron(&dur));
            println!("RON SystemTime: {}", crate::SerRon::serialize_ron(&time));
        }
    }
}

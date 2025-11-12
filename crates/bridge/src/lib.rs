#![allow(clippy::unnecessary_fallible_conversions)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::borrowed_box)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::ptr_arg)]
#![allow(unused_variables)]

pub mod logic;
pub mod providers;
pub mod repository;
pub mod router;

pub const DEFAULT_DATA_ENCRYPTION_KEY_ID: &str = "default";

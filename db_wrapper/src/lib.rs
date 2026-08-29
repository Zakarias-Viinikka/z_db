uniffi::setup_scaffolding!();

#[cfg(feature = "android")]
pub mod android_mascot;

#[cfg(feature = "web")]
pub mod web_mascot;

#[cfg(feature = "normal")]
pub mod mascot;

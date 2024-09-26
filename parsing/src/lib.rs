//! This crate contains the code dedicated to parsing the various databases.

pub mod bugcheck;
pub mod cpu;
pub mod pcie;
pub mod usb;

/// Because the error that nom uses is rather lengthy and unintuitive, it's defined here
/// to simplify handling
pub(crate) type NomError<'a> = nom::Err<nom::error::Error<&'a str>>;

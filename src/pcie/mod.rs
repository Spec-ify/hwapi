use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::char;
use nom::sequence::{delimited, preceded};
use nom::IResult;

// the input file was obtained from https://pci-ids.ucw.cz/
const FILE_INPUT: &str = include_str!("./pci.ids.txt");

/// Vendors are at the root of the file
pub struct Vendor {
    pub id: String,
    pub name: String,
    pub devices: Vec<Device>
}

/// Devices are placed directly under the relevant [Vendor] in the tree,
/// and are marked with one tab before, the device ID, then two spaces and the device name
pub struct Device {
    pub id: String,
    pub name: String,
    pub subsystems: Vec<Subsystem>
}

/// Subsystems are placed directly under the relevant [Device] in the tree,
/// and are marked with two tabs before, the [Vendor] ID, a space, then the subsystem ID,
/// then two spaces, then the name of the subsystem
pub struct Subsystem {
    pub id: String,
    pub name: String,
}
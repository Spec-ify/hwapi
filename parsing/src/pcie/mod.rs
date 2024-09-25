use crate::NomError;
use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::char;
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;
// https://stackoverflow.com/a/70552843
// this library is used for a very fast hashmap implementation because we're not worried about DOS attacks
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;

pub type PcieDeviceInfo = (Option<Vendor>, Option<Device>, Option<Subsystem>);

// the input file was obtained from https://pci-ids.ucw.cz/
const FILE_INPUT: &str = include_str!("./pci.ids.txt");

/// Vendors are at the root of the file
#[derive(PartialEq, Debug, Clone)]
pub struct Vendor {
    pub id: u16,
    pub name: String,
    pub devices: HashMap<u16, Device, BuildNoHashHasher<u16>>,
}

/// Devices are placed directly under the relevant [Vendor] in the tree,
/// and are marked with one tab before, the device ID, then two spaces and the device name
#[derive(PartialEq, Debug, Clone)]
pub struct Device {
    pub id: u16,
    pub name: String,
    pub subsystems: Vec<Subsystem>,
}

/// Subsystems are placed directly under the relevant [Device] in the tree,
/// and are marked with two tabs before, the [Vendor] ID, a space, then the subsystem ID,
/// then two spaces, then the name of the subsystem
#[derive(PartialEq, Debug, Clone)]
pub struct Subsystem {
    pub id: u16,
    pub name: String,
}

/// An interface for fetching and storing pcie devices
#[derive(Clone)]
pub struct PcieCache {
    /// A list of vendors, where each vendor contains associated devices and subsystems
    vendors: HashMap<u16, Vendor, BuildNoHashHasher<u16>>,
}

impl PcieCache {
    /// create a new pcie cache and parse the database into memory
    pub fn new() -> Self {
        let mut vendors: HashMap<u16, Vendor, BuildNoHashHasher<u16>> =
            HashMap::with_capacity_and_hasher(512, BuildNoHashHasher::default());
        for vendor in parse_pcie_db().unwrap() {
            vendors.insert(vendor.id, vendor);
        }
        // cut down on those unnecessary allocations again (1gb vps life)
        vendors.shrink_to_fit();
        Self { vendors }
    }

    #[tracing::instrument(name = "pcie_lookup", skip(self))]
    pub fn find<'a>(&'a self, input: &'a str) -> Result<PcieDeviceInfo, NomError> {
        let parsed_identifier = parse_device_identifier(input)?;
        // search for a vendor
        let vendor = self.vendors.get(&parsed_identifier.0);

        // if one were looking for even more performance, subsystem and device search should ideally also be done with a hashmap
        let mut device: Option<&Device> = None;
        if let Some(ven) = vendor {
            // a lot of these memory allocations are entirely necessary, just return a reference and figure out lifetimes
            device = ven.devices.get(&parsed_identifier.1);
        }

        let mut subsystem: Option<Subsystem> = None;
        if let Some(dev) = device {
            subsystem = dev
                .subsystems
                .iter()
                .find(|s| s.id == parsed_identifier.1)
                .cloned();
        }
        Ok((vendor.cloned(), device.cloned(), subsystem))
    }
}

impl Default for PcieCache {
    fn default() -> Self {
        Self::new()
    }
}

/// This function searches the input string for a vendor id, a product id, and optionally a subsystem ID
/// input strings are expected in the format of:
///
/// `PCI\VEN_10EC&DEV_8168&SUBSYS_86771043&REV_15\6&102E3ADF&0&0048020A`
///
/// Output is returned as a tuple of (`vendor`, `device`, `subsystem`)
fn parse_device_identifier(input: &str) -> Result<(u16, u16, Option<u16>), NomError<'_>> {
    // TODO: validate that ids are hex strings
    let vid_combinator = delimited(tag("PCI\\VEN_"), take(4_u8), char('&'))(input)?;
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/identifiers-for-pci-devices
    let did_combinator = preceded(tag("DEV_"), take(4_u8))(vid_combinator.0)?;
    let mut ssid: Option<&str> = None;
    if did_combinator.0.starts_with("&SU") {
        let ssid_combinator = preceded(tag("&SUBSYS_"), take(4_u8))(did_combinator.0)?;
        ssid = Some(ssid_combinator.1);
    }

    Ok((
        u16::from_str_radix(vid_combinator.1, 16).unwrap(),
        u16::from_str_radix(did_combinator.1, 16).unwrap(),
        ssid.map(|s| u16::from_str_radix(s, 16).unwrap()),
    ))
}

/// Read the database from the file into memory
fn parse_pcie_db() -> Result<Vec<Vendor>, NomError<'static>> {
    let header_combinator = read_header(FILE_INPUT)?;
    // this is filled up as the db is parsed
    let mut output: Vec<Vendor> = Vec::with_capacity(512);
    let mut iterated_output = read_vendor(header_combinator.0);
    while let Ok(ref section_output) = iterated_output {
        output.push(section_output.1.clone());
        iterated_output = read_vendor(section_output.0);
    }

    Ok(output)
}

// read the commented header of the input up until the start of the actual list
fn read_header(input: &str) -> IResult<&str, &str> {
    take_until("0001 ")(input)
}

// read a single vendor block and all associated devices/subsystems from the input
fn read_vendor(input: &str) -> IResult<&str, Vendor> {
    let vid_combinator = terminated(take(4_u8), tag("  "))(input)?;
    let vid = vid_combinator.1;
    let vname_combinator = terminated(take_until("\n"), char('\n'))(vid_combinator.0)?;
    let vname = vname_combinator.1;
    // read until the next line doesn't start with a tab
    let mut devices: HashMap<u16, Device, BuildNoHashHasher<u16>> =
        HashMap::with_hasher(BuildNoHashHasher::default());
    let mut iterated_output = read_device(vname_combinator.0);
    let mut leftover = vname_combinator.0;
    loop {
        if let Ok(combinator_output) = iterated_output {
            leftover = combinator_output.0;
            devices.insert(combinator_output.1.id, combinator_output.1);
            iterated_output = read_device(combinator_output.0);
        } else {
            // Some lines have comments, handle those here, this is assuming the next line is indented
            if leftover.starts_with('#') {
                leftover = preceded(take_until("\n"), char('\n'))(leftover)?.0;
                iterated_output = read_device(leftover);
                continue;
            }
            break;
        }
    }

    Ok((
        leftover,
        Vendor {
            id: u16::from_str_radix(vid, 16).unwrap(),
            name: String::from(vname),
            devices,
        },
    ))
}

// read a single device and all associated subsystems (if applicable) from the input
fn read_device(input: &str) -> IResult<&str, Device> {
    let did_combinator = preceded(char('\t'), take(4_usize))(input)?;
    let did = did_combinator.1;
    let dname_combinator = delimited(tag("  "), take_until("\n"), char('\n'))(did_combinator.0)?;

    // read until the next line doesn't start with a tab
    let mut subsystems: Vec<Subsystem> = Vec::new();
    let mut iterated_output = read_subsystem_line(dname_combinator.0);
    // this is so that we can actually return the leftover of the iterated parsing
    let mut leftover = dname_combinator.0;
    loop {
        if let Ok(combinator_output) = iterated_output {
            leftover = combinator_output.0;
            subsystems.push(combinator_output.1);
            iterated_output = read_subsystem_line(combinator_output.0);
        } else {
            // Some lines have comments, handle those here, this is assuming the next line is indented
            if leftover.starts_with('#') {
                leftover = preceded(take_until("\n"), char('\n'))(leftover)?.0;
                iterated_output = read_subsystem_line(leftover);
                continue;
            }
            break;
        }
    }

    Ok((
        leftover,
        Device {
            id: u16::from_str_radix(did, 16).unwrap(),
            name: String::from(dname_combinator.1),
            subsystems,
        },
    ))
}

// read a single subsystem from the input
fn read_subsystem_line(input: &str) -> IResult<&str, Subsystem> {
    // subsystems in the file are identified by two tabs, the vendor ID, a space, the subsystem id,
    // two spaces, then the name.

    // the vid is not needed, but might as well break the parsing down in steps
    let vid_combinator = delimited(tag("\t\t"), take(4_u8), char(' '))(input)?;

    // subsystem id
    let ssid_combinator = terminated(take(4_u8), tag("  "))(vid_combinator.0)?;
    let ss_name_combinator = terminated(take_until("\n"), char('\n'))(ssid_combinator.0)?;
    Ok((
        ss_name_combinator.0,
        Subsystem {
            id: u16::from_str_radix(ssid_combinator.1, 16).unwrap(),
            name: String::from(ss_name_combinator.1),
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nohash_hasher::BuildNoHashHasher;

    use crate::pcie::{
        parse_device_identifier, read_device, read_subsystem_line, read_vendor, Device, Subsystem,
        Vendor,
    };

    use super::{parse_pcie_db, read_header, PcieCache};

    #[test]
    fn basic_read_header() {
        let mock_header = "# blah blah\n# foo bar\n\n0001 Safenet";
        assert_eq!(
            read_header(mock_header),
            Ok(("0001 Safenet", "# blah blah\n# foo bar\n\n"))
        )
    }

    #[test]
    fn basic_read_subsystem_line() {
        let mock_subsystem_line = "\t\tabcd 0001  foo bar\nbat";
        assert_eq!(
            read_subsystem_line(mock_subsystem_line),
            Ok((
                "bat",
                Subsystem {
                    id: 0x0001,
                    name: String::from("foo bar")
                }
            ))
        );
    }

    #[test]
    fn basic_read_device() {
        let mock_device_no_subsys = "\t0001  foo bar\n\t0002";
        assert_eq!(
            read_device(mock_device_no_subsys),
            Ok((
                "\t0002",
                Device {
                    id: 0x0001,
                    name: String::from("foo bar"),
                    subsystems: vec![]
                }
            ))
        );

        let mock_device_w_subsys = "\t0001  foo bar\n\t\t000a 8008  subsys\n0002";
        assert_eq!(
            read_device(mock_device_w_subsys),
            Ok((
                "0002",
                Device {
                    id: 0x0001,
                    name: String::from("foo bar"),
                    subsystems: vec![Subsystem {
                        id: 0x8008,
                        name: String::from("subsys")
                    }],
                }
            ))
        );

        let mock_device_w_comment = "\t0001  foo bar\n# bat\n\t\t000a 8008  subsys\n0002";
        assert_eq!(
            read_device(mock_device_w_comment),
            Ok((
                "0002",
                Device {
                    id: 0x0001,
                    name: String::from("foo bar"),
                    subsystems: vec![Subsystem {
                        id: 0x8008,
                        name: String::from("subsys")
                    }],
                }
            ))
        );
    }

    #[test]
    fn basic_read_vendor() {
        let mock_vendor = "0001  foo\n\t000a  bar\n0002";
        let mut mock_devices: HashMap<u16, Device, nohash_hasher::BuildNoHashHasher<u16>> =
            HashMap::with_hasher(BuildNoHashHasher::default());
        mock_devices.insert(
            0x00a,
            Device {
                id: 0x000a,
                name: String::from("bar"),
                subsystems: vec![],
            },
        );
        assert_eq!(
            read_vendor(mock_vendor),
            Ok((
                "0002",
                Vendor {
                    id: 0x0001,
                    name: String::from("foo"),
                    devices: mock_devices
                }
            ))
        )
    }

    #[test]
    fn basic_parse_db() {
        // basically make sure a panic doesn't occur during parsing
        parse_pcie_db().unwrap();
    }

    #[test]
    fn basic_parse_device_identifier() {
        // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/identifiers-for-pci-devices
        assert_eq!(
            parse_device_identifier("PCI\\VEN_1234&DEV_5678&SUBSYS_91230000&REV_00"),
            Ok((0x1234, 0x5678, Some(0x9123)))
        );
        assert_eq!(
            parse_device_identifier("PCI\\VEN_1234&DEV_5678&SUBSYS_91230000"),
            Ok((0x1234, 0x5678, Some(0x9123)))
        );
        assert_eq!(
            parse_device_identifier("PCI\\VEN_1234&DEV_5678&REV_00"),
            Ok((0x1234, 0x5678, None))
        );
        assert_eq!(
            parse_device_identifier("PCI\\VEN_1234&DEV_5678"),
            Ok((0x1234, 0x5678, None))
        );
        assert_eq!(
            parse_device_identifier("PCI\\VEN_1234&DEV_5678&CC_112200"),
            Ok((0x1234, 0x5678, None))
        );
        assert_eq!(
            parse_device_identifier("PCI\\VEN_1234&DEV_5678&CC_1122"),
            Ok((0x1234, 0x5678, None))
        );
    }

    #[test]
    fn basic_find_device() {
        let cache = PcieCache::new();
        assert_eq!(
            cache
                .find("PCI\\VEN_1022&DEV_1633&SUBSYS_14531022&REV_00\\3&2411E6FE&1&09")
                .map(|t| t.1)
                .unwrap(),
            Some(Device {
                id: 5683,
                name: String::from("Renoir PCIe GPP Bridge"),
                subsystems: vec![],
            },),
        );
    }
}

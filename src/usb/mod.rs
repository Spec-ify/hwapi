use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::char;
use nom::sequence::{delimited, preceded};
use nom::IResult;

use crate::NomError;

// The input file was obtained from http://www.linux-usb.org/
// note: only vendors and devices are currently read from the file, there's extra crap at the bottom that might be useful
// This file contains one or two invalid utf 8 characters, so it's parsed slightly differently
const INPUT_FILE: &[u8] = include_bytes!("usb.ids.txt");

#[derive(Clone, Debug, PartialEq)]
pub struct Vendor {
    pub id: u16,
    pub name: String,
    pub devices: Vec<Device>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub id: u16,
    pub name: String,
}

#[derive(Clone)]
pub struct UsbCache {
    vendors: Vec<Vendor>,
}

impl UsbCache {
    pub fn new() -> Self {
        Self {
            vendors: parse_usb_db(),
        }
    }

    /// Search the cache for the provided input string, returning the found device info, if it exists. If the `Option<Vendor>` is `None`,
    /// you can assume that the device info will also be `None`.
    ///
    /// TODO: this function calls unwrap on a very fallible function, change function
    /// to return a Result, you could then make it so that vendor and device aren't options
    pub fn find<'a>(
        &'a self,
        input: &'a str,
    ) -> Result<(Option<Vendor>, Option<Device>), NomError<'a>> {
        let parsed_identifier = parse_device_identifier(input)?;
        // first search for a vendor
        let matching_vendor = self
            .vendors
            .iter()
            .filter(|ven| ven.id == parsed_identifier.0)
            .nth(0);

        let mut device: Option<Device> = None;
        if let Some(vendor) = matching_vendor {
            device = vendor
                .devices
                .iter()
                .filter(|dev| dev.id == parsed_identifier.1)
                .nth(0)
                .cloned();
        }

        Ok((matching_vendor.cloned(), device))
    }
}

/// This function searches the input string for a vendor id (vid) and product id (pid).
/// Input strings in the form of `USB\VID_1234&PID_5678\9479493` are assumed.
/// It returns a tuple, where the first value is the vendor id, and the second is the product id. This tuple contains substrings of the initial input string,
/// so handle lifetimes accordingly.
fn parse_device_identifier(device_string: &str) -> Result<(u16, u16), NomError> {
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/standard-usb-identifiers
    // TODO: this does not fully support all formats of usb device identifiers
    let vid_combinator = delimited(tag("USB\\VID_"), take(4 as u8), take(1 as u8))(device_string)?;
    let pid_combinator = preceded(tag("PID_"), take(4 as u8))(vid_combinator.0)?;
    Ok((
        u16::from_str_radix(vid_combinator.1, 16).unwrap(),
        u16::from_str_radix(pid_combinator.1, 16).unwrap(),
    ))
}

fn parse_usb_db() -> Vec<Vendor> {
    // this is kind of awful, but there's an invalid utf 8 character at byte 703748,
    // so we just stop before then, because it's past the section we care about
    let file_as_str = std::str::from_utf8(&INPUT_FILE[0..703_748]).unwrap();
    let header_combinator_output = read_header(file_as_str).unwrap();
    let mut output: Vec<Vendor> = Vec::with_capacity(1024);
    let mut iterated_output = read_vendor(header_combinator_output.0);
    loop {
        if let Ok(ref section_output) = iterated_output {
            output.push(section_output.1.clone());
            iterated_output = read_vendor(section_output.0);
        } else {
            break;
        }
    }
    output
}

/// read the commented header up until the
/// start of the actual list. The `input` portion of the returned
/// tuple is the only part expected to be used, the header can be discarded
fn read_header(input: &str) -> IResult<&str, &str> {
    // this is making the assumption that the list will always start with vendor 001
    take_until("0001")(input)
}

/// This combinator reads a a vendor and all of the associated ids from the file
fn read_vendor(input: &str) -> IResult<&str, Vendor> {
    // read the vendor id and vendor name
    let vid_combinator_output = take(4_u8)(input)?;
    let vid = vid_combinator_output.1;
    let vname_combinator =
        delimited(tag("  "), take_until("\n"), char('\n'))(vid_combinator_output.0)?;
    let vname = vname_combinator.1;
    // read until the next line doesn't start with a tab
    let mut devices: Vec<Device> = Vec::new();
    let mut iterated_output = read_device_line(vname_combinator.0);
    // this is so that we can actually return the leftover of the iterated parsing
    let mut leftover = vname_combinator.0;
    loop {
        if let Ok(combinator_output) = iterated_output {
            leftover = combinator_output.0;
            devices.push(combinator_output.1);
            iterated_output = read_device_line(combinator_output.0);
        } else {
            // Some lines have comments, handle those here, this is assuming the next line is indented
            if leftover.starts_with("#") {
                leftover = take_until("\t")(leftover)?.0;
                iterated_output = read_device_line(leftover);
                continue;
            }
            break;
        }
    }

    Ok((
        leftover,
        Vendor {
            id: u16::from_str_radix(vid, 16).unwrap(),
            name: vname.to_string(),
            devices,
        },
    ))
}

/// This combinator reads a single device line from the input, if it is formed correctly
fn read_device_line(input: &str) -> IResult<&str, Device> {
    let combinator_output = delimited(char('\t'), take_until("\n"), char('\n'))(input)?;
    // read the device id and device name
    let did_combinator_output = take(4 as u8)(combinator_output.1)?;
    let dname = take(2 as u8)(did_combinator_output.0)?.0;
    Ok((
        combinator_output.0,
        Device {
            id: u16::from_str_radix(did_combinator_output.1, 16).unwrap(),
            name: String::from(dname),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse_device_identifier;
    use super::{parse_usb_db, read_vendor};
    use super::{read_device_line, read_header, Device, Vendor};

    #[test]
    fn basic_parse_device_string() {
        let mock_device_string = "USB\\VID_1234&PID_5678\\9479493";
        assert_eq!(
            parse_device_identifier(mock_device_string),
            Ok((0x1234, 0x5678))
        );
    }

    #[test]
    fn basic_read_header() {
        let mock_header = "#\tinterface  interface_name\t\t<-- two tabs\n\n0001";
        assert_eq!(
            read_header(mock_header),
            Ok(("0001", "#\tinterface  interface_name\t\t<-- two tabs\n\n"))
        );
    }

    #[test]
    fn basic_read_vendor() {
        let mock_section = "1234  vendor_name\n\t5678  device_name\n9123";
        let expected_output = Vendor {
            id: 0x1234,
            name: String::from("vendor_name"),
            devices: vec![Device {
                id: 0x5678,
                name: String::from("device_name"),
            }],
        };
        assert_eq!(read_vendor(mock_section), Ok(("9123", expected_output)));
    }

    #[test]
    fn read_section_no_devices() {
        let mock_section = "1234  vendor_name\n5678";
        let expected_output = Vendor {
            id: 0x1234,
            name: String::from("vendor_name"),
            devices: vec![],
        };
        assert_eq!(read_vendor(mock_section), Ok(("5678", expected_output)));
        // first make sure we can read a normal device without issue
        let mock_device_entry = "\t1234  foo bar\n4567";
        assert_eq!(
            read_device_line(mock_device_entry),
            Ok((
                "4567",
                Device {
                    id: 0x1234,
                    name: String::from("foo bar")
                }
            ))
        );
    }

    #[test]
    fn basic_parse_usbs() {
        parse_usb_db();
    }
}

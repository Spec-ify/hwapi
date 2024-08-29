# HWAPI
This API provides a backend for fetching information about hardware.

Current information fetched includes:
- CPU info (Intel ARK, AMD Product Database)
- USB info (VID/PID mapping)
- PCIe info (VID/PID/SUBSYS mapping)

## Project layout
The code is organized into 4 separate crates:
- `parsing`: The code for parsing the raw databases into Rust abstractions.
- `databases`: The interfaces for fast lookup from those databases.
- `handlers`: The HTTP endpoint code
- `server`: The binary and runtime specific details

## Building
Building this project requires installing the rust toolchain.

To build the project (development):
```
cargo build
```

To run the project (development):
```
cargo run
```
This will start a server on port 3000.

To build a release executable:
```
cargo build --release
```
The resulting executable will be located in `./target/release/`.

## Testing
Testing also requires the rust toolchain installed.

To _test_ the project:
```
cargo test
```

To _profile_ the project, you'll need `cargo flamegraph` installed (`cargo install flamegraph`):
```
cargo flamegraph --root --unit-test -- [TEST_NAME]
```
Where `[TEST_NAME]` is the test you want to profile. There are other ways to profile different parts of the code, but you can figure that out yourself.

## Configuration
By default, the application listens on port `3000`, although that can be configured by setting the `HWAPI_PORT` environment variable or by passing `-p`/`--port`.

## Endpoints
### CPU
To interact with the CPU API, submit a `GET` request to `/api/cpus/?name=[CPU_NAME]`, where `[CPU_NAME]` is the URL encoded name of the cpu.

This endpoint does not guarantee the correctness of the model returned, it will always attempt to return a model.

Here's an example curl request:
```
curl "http://localhost:3000/api/cpus/?name=Intel%20Core%20i9-9900k"
```

### USB
To interact with the USB API, submit a `GET` request to `/api/usbs/?identifier=[USB_IDENTIFIER_STRING]`, where `[USB_IDENTIFIER_STRING]` is a valid [USB identifier](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/identifiers-for-usb-devices).

The endpoint will return a structure that looks like this:
```json
{
    "vendor": "string",
    "device": "string | null",
}
```

Responses:<br>
| Code | Meaning |
| -- | -- |
| `404` | No USB info was found with the given identifier |


Here's an example curl request:
```
curl "http://127.0.0.1:3000/api/usbs/?identifier=USB%5CVID_1532%26PID_0084%26MI_03%5C6%2638C0FA5D%260%260003"
```

For bulk processing, you may submit a `POST` request to the same endpoint with a `Content-Type` of `application/json` and a payload containing an array of USB device identifier strings.

The endpoint will return an array of objects (same shape as the `GET` request), or if an identifier string was unable to be processed successfully, `null` will substitute in the response.

Here's an example curl request:
```
curl -X POST http://127.0.0.1:3000/api/usbs/ -H "Content-Type: application/json" -d '["USB\\\\VID_1038&PID_1729\\\\6&28B2
9415&0&4","USB\\\\VID_413C&PID_2105\\\\6&3B66B33D&0&9","USB\\\\ROOT_HUB30\\\\5&381F2DE&0&0","USB\\\\VID_1038&PID_1729&MI_00\\\\7&381E8103&0&0000","USB\\\\VID_0B05&PID_184C\\\\123456","US
B\\\\VID_1038&PID_1729&MI_01\\\\7&381E8103&0&0001","USB\\\\ROOT_HUB30\\\\5&3B7A03C3&0&0"]'
```

And here's an example response (truncated):
```json
[
   {
      "vendor":"SteelSeries ApS",
      "device":null
   },
   {
      "vendor":"Dell Computer Corp.",
      "device":"Model L100 Keyboard"
   },
   null
]
```

### PCIe
To interact with the PCIe API, submit a `GET` request to `/api/pcie/?identifier=[PCIE_IDENTIFIER_STRING]`, where `[PCIE_IDENTIFIER_STRING]` is a valid [PCIe identifier](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/identifiers-for-pci-devices).

The endpoint will return a structure that looks like this:
```json
{
    "vendor": "string",
    "device": "string | null",
    "subsystem": "string | null"
}
```

Responses:<br>
| Code | Meaning |
| -- | -- |
| `404` | No PCIe info was found with the given identifier |


Here's an example curl request:
```
curl "http://127.0.0.1:3000/api/pcie/?identifier=PCI%5CVEN_8086%26DEV_7A4D%26SUBSYS_00000000%26REV_11%5C3%2611583659%260%26A9%0A"
```

For bulk processing, you may submit a `POST` request to the same endpoint with a `Content-Type` of `application/json` and a payload containing an array of PCIe device identifier strings.

The endpoint will return an array of objects (same shape as the `GET` request), or if an identifier string was unable to be processed successfully, `null` will substitute in the response.

Here's an example curl request:
```
curl -X POST http://127.0.0.1:3000/api/pcie/ -H "Content-Type: application/json" -d '["PCI\\\\VEN_1022&DEV_43B4&SUBSYS_33
061B21&REV_02\\\\5&3B34128B&0&30020B","PCI\\\\VEN_1022&DEV_1444&SUBSYS_00000000&REV_00\\\\3&11583659&0&C4","PCI\\\\VEN_1022&DEV_43BC&SUBSYS_11421B21&REV_02\\\\4&2C18E2E3&0&000B","PCI\\\\
VEN_1022&DEV_43B4&SUBSYS_33061B21&REV_02\\\\5&3B34128B&0&28020B","PCI\\\\VEN_1022&DEV_1441&SUBSYS_00000000&REV_00\\\\3&11583659&0&C1","PCI\\\\VEN_1022&DEV_43B4&SUBSYS_33061B21&REV_02\\\\
5&3B34128B&0&38020B"]'
```

And here's example response (truncated):
```
[
   {
      "vendor":"Advanced Micro Devices, Inc. [AMD]",
      "device":"300 Series Chipset PCIe Port",
      "subsystem":null
   },
   {
      "vendor":"Advanced Micro Devices, Inc. [AMD]",
      "device":"Matisse/Vermeer Data Fabric: Device 18h; Function 4",
      "subsystem":null
   },
]
```
# HWAPI
This API provides a backend for fetching information about hardware.

Current information fetched includes:
- CPU info (Intel ARK, AMD Product Database)
- USB info (VID/PID mapping)

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

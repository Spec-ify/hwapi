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

To test the project:
```
cargo test
```

## Configuration
By default, the application listens on port `3000`, although that can be configured by setting the `HWAPI_PORT` environment variable or by passing `-p`/`--port`.

## Endpoints
To interact with the cpu api, submit a `GET` request to `/api/cpus/?name=[CPU_NAME]`, where `[CPU_NAME]` is the HTTP encoded name of the cpu.


This endpoint does not guarantee the correctness of the model returned, it will always attempt to return a model.

Here's an example curl request:
```
curl "http://localhost:3000/api/cpus/?name=Intel%20Core%20i9-9900k"
```

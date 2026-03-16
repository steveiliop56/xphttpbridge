# XPHTTPBridge

A lightweight [X-Plane 11](https://www.x-plane.com/) plugin written in Rust that exposes an HTTP API for reading and writing X-Plane DataRefs.

> [!WARNING]
> An LLM was partially invloved in the development of this plugin since I don't write Rust often. It was only used for debugging type errors and generating documentation.

## Features

- Read and write X-Plane DataRefs over HTTP
- Supports single and array DataRef values
- Browse all available DataRefs via the API
- Configurable listen address and port

## Requirements

- [Rust](https://rustup.rs/) (edition 2024)
- X-Plane 11
- `make`

## Building

Clone the repository and run:

```sh
make build
```

This will:
1. Compile the plugin with `cargo build`
2. Create the `XPHTTPBridge/` plugin directory
3. Copy the compiled library, config, license, and README into it

The output plugin folder will be placed at `XPHTTPBridge/` in the project root.

## Installation

Copy the `XPHTTPBridge/` folder into your X-Plane plugins directory:

```
X-Plane 11/Resources/plugins/XPHTTPBridge/
```

The final layout should look like this:

```
X-Plane 11/Resources/plugins/XPHTTPBridge/
├── 64/
│   ├── lin.xpl   (Linux)
│   ├── win.xpl   (Windows)
│   └── mac.xpl   (macOS)
├── config.ini
├── LICENSE
└── README.md
```

Start X-Plane and the plugin will load automatically, starting the HTTP server on the configured address and port.

> [!WARNING]
> Do not rename the plugin folder as it will break the plugin's path resolution.

## Configuration

The plugin reads its configuration from `config.ini` inside the plugin folder. A reference config is provided at `config.example.ini`:

```ini
[server]
port=49000
address=127.0.0.1
```

| Key       | Default     | Description                          |
|-----------|-------------|--------------------------------------|
| `port`    | `49000`     | The port the HTTP server listens on  |
| `address` | `127.0.0.1` | The address the HTTP server binds to |

To expose the server on your local network, set `address=0.0.0.0`.

## API Reference

All endpoints are prefixed with `/api/v1`. Responses are JSON.

### `GET /api/v1/healthz`

Returns a health check response.

**Response**
```json
{ "status": 200, "message": "OK" }
```

---

### `GET /api/v1/refs/all`

Returns a list of all known DataRefs.

**Query Parameters**

| Parameter    | Required | Description                                  |
|--------------|----------|----------------------------------------------|
| `fetch_size` | No       | Limit the number of DataRefs returned        |

**Response**
```json
{ "status": 200, "message": "OK", "refs": [ ... ] }
```

---

### `GET /api/v1/refs/value/get`

Get the scalar value of a DataRef.

**Query Parameters**

| Parameter  | Required | Description           |
|------------|----------|-----------------------|
| `ref_name` | Yes      | Full DataRef path     |

**Response**
```json
{ "status": 200, "message": "OK", "ref_name": "sim/cockpit/autopilot/heading_mag", "ref_value": 270.0 }
```

---

### `POST /api/v1/refs/value/set`

Set the scalar value of a DataRef.

**Query Parameters**

| Parameter  | Required | Description       |
|------------|----------|-------------------|
| `ref_name` | Yes      | Full DataRef path |

**Request Body**
```json
{ "ref_value": 270.0 }
```

**Response**
```json
{ "status": 200, "message": "OK", "ref_name": "sim/cockpit/autopilot/heading_mag", "ref_value": 270.0 }
```

---

### `GET /api/v1/refs/values/get`

Get the array values of a DataRef.

**Query Parameters**

| Parameter  | Required | Description       |
|------------|----------|-------------------|
| `ref_name` | Yes      | Full DataRef path |

**Response**
```json
{ "status": 200, "message": "OK", "ref_name": "sim/some/array_ref", "ref_values": [ ... ] }
```

---

### `POST /api/v1/refs/values/set`

Set the array values of a DataRef.

**Query Parameters**

| Parameter  | Required | Description       |
|------------|----------|-------------------|
| `ref_name` | Yes      | Full DataRef path |

**Request Body**
```json
{ "ref_values": [ 1.0, 2.0, 3.0 ] }
```

**Response**
```json
{ "status": 200, "message": "OK", "ref_name": "sim/some/array_ref", "ref_values": [ 1.0, 2.0, 3.0 ] }
```

---

## License

Copyright 2026 steveiliop56. Licensed under the [MIT License](LICENSE).

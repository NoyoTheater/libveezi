# libveezi

A Rust crate to work with the [Veezi API](https://api.us.veezi.com/).

## Features

-   Full coverage of Veezi API endpoints
-   Asynchronous requests using `reqwest`
-   Strongly typed data structures with `serde` for easy serialization/deserialization

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libveezi = "0.1.0"
```

## Usage

```rust
use libveezi::VeeziClient;
use libveezi::Session;

let client = VeeziClient::new("api.us.veezi.com", "your_api_key");
let sessions: Vec<Session> = client.get_sessions().await?;
```

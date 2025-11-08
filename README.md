# libveezi

A Rust crate to work with the [Veezi API](https://api.us.veezi.com/).

## Features

-   Full coverage of Veezi API endpoints
-   Asynchronous requests using `reqwest`
-   Strongly typed data structures with `serde` for easy serialization/deserialization
-   Convenient helper methods for common operations
-   Built-in caching support with configurable TTL

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libveezi = "0.4.0"
```

## Usage

### Basic Usage

```rust,no_run
use libveezi::client::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default caching
    let client = ClientBuilder::new("https://api.us.veezi.com", "your_api_key".to_string())
        .with_default_caching()
        .build()?;

    // Get all sessions
    let sessions = client.list_sessions().await?;
    println!("Found {} sessions", sessions.len());

    Ok(())
}
```

### Convenience Methods

The library provides many convenience methods to make common tasks easier:

```rust,no_run
use libveezi::client::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new("https://api.us.veezi.com", "your_api_key".to_string())
        .with_default_caching()
        .build()?;

    // Get sessions for today
    let today_sessions = client.list_sessions_today().await?;

    // Get only sessions that are open for sales
    let available_sessions = client.list_sessions_open_for_sales().await?;

    // Get active films
    let active_films = client.list_active_films().await?;

    // Get films currently showing (with sessions scheduled)
    let now_showing = client.list_films_now_showing().await?;

    Ok(())
}
```

### Filtering and Sorting Sessions

```rust,no_run
use libveezi::client::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new("https://api.us.veezi.com", "your_api_key".to_string())
        .with_default_caching()
        .build()?;

    // Get all sessions, filter, and sort them
    let sessions = client.list_sessions().await?
        .filter_today()
        .filter_open_for_sales()
        .filter_with_available_seats()
        .sort_by_start_time();

    // Get statistics
    println!("Total available seats: {}", sessions.total_available_seats());
    println!("Average seats sold: {}", sessions.average_sold_seats());

    // Group by date
    for (date, day_sessions) in sessions.group_by_date() {
        println!("{}: {} sessions", date, day_sessions.len());
    }

    Ok(())
}
```

### Working with Films

```rust,no_run
use libveezi::client::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new("https://api.us.veezi.com", "your_api_key".to_string())
        .with_default_caching()
        .build()?;

    let films = client.list_active_films().await?;
    
    for film in films {
        println!("Title: {}", film.title);
        println!("Duration: {}", film.formatted_duration());
        println!("Rating: {}", film.rating_display());
        println!("Actors: {}", film.actors_formatted());
        println!("Directors: {}", film.directors_formatted());
        println!("Is 3D: {}", film.is_3d());
        println!("---");
    }

    Ok(())
}
```


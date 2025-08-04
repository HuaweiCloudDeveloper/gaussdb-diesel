# Diesel-GaussDB

A complete GaussDB backend implementation for the Diesel ORM framework.

## Overview

Diesel-GaussDB provides a fully-featured GaussDB database backend for Diesel, enabling Rust applications to work with GaussDB databases using Diesel's type-safe query builder. This implementation includes a complete backend, query builder, type system, and connection management.

## Features

- **Complete Diesel Backend**: Full implementation of all Diesel backend traits
- **PostgreSQL-Compatible SQL**: Generates PostgreSQL-compatible SQL optimized for GaussDB
- **Type Safety**: Comprehensive type mapping between Rust and GaussDB types
- **Complex Types**: Support for PostgreSQL-compatible arrays and planned range types
- **Real Database Connectivity**: Uses the `gaussdb` crate for authentic GaussDB connections
- **Feature-based Compilation**: Optional real database integration with mock fallback
- **Query Builder**: Custom query builder with proper identifier escaping and parameter binding

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
diesel = "2.2"
diesel-gaussdb = "0.1.0-alpha"

# For real GaussDB connectivity
[features]
gaussdb = ["diesel-gaussdb/gaussdb"]
```

## Usage

### Basic Setup

```rust
use diesel::prelude::*;
use diesel_gaussdb::{GaussDB, GaussDBConnection};

// Connect to GaussDB
let database_url = "gaussdb://user:password@localhost:5432/database";
let mut connection = GaussDBConnection::establish(database_url)?;
```

### Define Your Schema

```rust
diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(GaussDB))]
struct User {
    id: i32,
    name: String,
    email: String,
}
```

### Perform Queries

```rust
// Insert data
diesel::insert_into(users::table)
    .values((
        users::name.eq("John Doe"),
        users::email.eq("john@example.com"),
    ))
    .execute(&mut connection)?;

// Query data
let all_users = users::table
    .select(User::as_select())
    .load(&mut connection)?;
```

## Supported Types

The GaussDB backend supports comprehensive type mapping:

| Rust Type | GaussDB Type | Diesel Type |
|-----------|--------------|-------------|
| `i16` | `SMALLINT` | `SmallInt` |
| `i32` | `INTEGER` | `Integer` |
| `i64` | `BIGINT` | `BigInt` |
| `f32` | `REAL` | `Float` |
| `f64` | `DOUBLE PRECISION` | `Double` |
| `String` | `TEXT` | `Text` |
| `Vec<u8>` | `BYTEA` | `Binary` |
| `bool` | `BOOLEAN` | `Bool` |
| `Vec<T>` | `T[]` | `Array<T>` |

### Complex Types

The backend supports PostgreSQL-compatible complex types:

- **Arrays**: One-dimensional arrays of basic types (`Vec<T>` ↔ `Array<T>`)
- **Range Types**: Planned for future implementation

```rust
// Array usage example
diesel::table! {
    posts (id) {
        id -> Integer,
        tags -> Array<Text>,
        scores -> Array<Integer>,
    }
}

let post_tags: Vec<String> = posts::table
    .select(posts::tags)
    .first(&mut connection)?;
```

For detailed information about complex types, see [COMPLEX_TYPES.md](COMPLEX_TYPES.md).

## Features

### `gaussdb` Feature

Enable real GaussDB connectivity:

```bash
cargo build --features gaussdb
```

Without this feature, a mock implementation is used for development and testing.

## Examples

```bash
# Run basic example
cargo run --example basic_usage

# Run with real GaussDB
cargo run --example basic_usage --features gaussdb
```

## Implementation Status

- [x] Complete Diesel Backend implementation
- [x] PostgreSQL-compatible query builder
- [x] Comprehensive type system
- [x] Complex types support (Arrays)
- [x] Connection management
- [x] Feature-based compilation
- [x] Mock implementation for testing
- [x] Real GaussDB connectivity
- [x] Comprehensive test suite
- [ ] Range types support (planned)
- [ ] Multi-dimensional arrays (planned)
- [ ] Array serialization (ToSql) (planned)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

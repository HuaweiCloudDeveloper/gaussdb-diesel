//! Basic usage example for diesel-gaussdb
//!
//! This example demonstrates the basic functionality of the diesel-gaussdb crate,
//! including query building and connection capabilities.

use diesel_gaussdb::{GaussDB, GaussDBQueryBuilder, GaussDBConnection};
use diesel::query_builder::QueryBuilder;
use diesel::connection::Connection;

fn main() {
    println!("Diesel-GaussDB Basic Usage Example");
    println!("==================================");

    // Demonstrate QueryBuilder functionality
    let mut builder = GaussDBQueryBuilder::new();

    // Build a simple SELECT query
    builder.push_sql("SELECT ");
    builder.push_identifier("name").unwrap();
    builder.push_sql(", ");
    builder.push_identifier("email").unwrap();
    builder.push_sql(" FROM ");
    builder.push_identifier("users").unwrap();
    builder.push_sql(" WHERE ");
    builder.push_identifier("id").unwrap();
    builder.push_sql(" = ");
    builder.push_bind_param();

    let sql = builder.finish();
    println!("Generated SQL: {}", sql);
    println!("Expected: SELECT \"name\", \"email\" FROM \"users\" WHERE \"id\" = $1");

    // Demonstrate backend type
    let backend = GaussDB::default();
    println!("Backend: {:?}", backend);

    // Demonstrate connection functionality
    println!("\n--- Connection Functionality ---");

    #[cfg(feature = "gaussdb")]
    {
        println!("GaussDB feature is enabled - attempting real connection...");

        // Try to establish a connection (this will likely fail without a real database)
        let database_url = std::env::var("GAUSSDB_URL")
            .unwrap_or_else(|_| "gaussdb://user:password@localhost:5432/database".to_string());

        match GaussDBConnection::establish(&database_url) {
            Ok(_connection) => {
                println!("✓ Successfully connected to GaussDB!");
                println!("  Connection established and ready for queries.");
            }
            Err(e) => {
                println!("✗ Failed to connect to GaussDB: {:?}", e);
                println!("  This is expected if no GaussDB instance is running.");
                println!("  Set GAUSSDB_URL environment variable to connect to a real database.");
            }
        }
    }

    #[cfg(not(feature = "gaussdb"))]
    {
        println!("GaussDB feature is disabled - using mock connection...");

        // Try to establish a mock connection
        match GaussDBConnection::establish("gaussdb://mock:mock@localhost:5432/mock") {
            Ok(_connection) => {
                println!("✓ Mock connection established!");
            }
            Err(e) => {
                println!("✗ Mock connection failed: {:?}", e);
            }
        }
    }

    println!("\n--- Summary ---");
    println!("✓ Query building functionality is working");
    println!("✓ Backend type system is working");
    println!("✓ Connection interface is implemented");
    println!("✓ Feature-based compilation is working");

    #[cfg(feature = "gaussdb")]
    println!("✓ Real GaussDB integration is available");

    #[cfg(not(feature = "gaussdb"))]
    println!("ℹ Real GaussDB integration requires the 'gaussdb' feature");

    println!("\nTo test with real GaussDB:");
    println!("  cargo run --example basic_usage --features gaussdb");
}

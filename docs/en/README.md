# diesel-gaussdb English Documentation

Welcome to the English documentation for diesel-gaussdb! This comprehensive guide will help you get started with using diesel-gaussdb in your Rust applications.

## 📚 Documentation Structure

### 🚀 Getting Started
- [Quick Start Guide](guides/getting-started.md) - Get up and running quickly
- [Installation](guides/installation.md) - Installation and setup instructions
- [Configuration](guides/configuration.md) - Configuration options and best practices

### 📖 User Guides
- [Database Connections](guides/connections.md) - Managing database connections
- [Query Building](guides/queries.md) - Building and executing queries
- [Transactions](guides/transactions.md) - Transaction management
- [Migrations](guides/migrations.md) - Database schema migrations
- [Testing](guides/testing.md) - Testing your database code
- [Performance](guides/performance.md) - Performance optimization
- [Best Practices](guides/best-practices.md) - Recommended patterns and practices
- [Troubleshooting](guides/troubleshooting.md) - Common issues and solutions

### 🔧 API Reference
- [Core Types](api/core-types.md) - Main types and traits
- [Connection API](api/connection.md) - Connection management
- [Query Builder API](api/query-builder.md) - Query construction
- [Type System](api/types.md) - Data type mappings
- [Error Handling](api/errors.md) - Error types and handling
- [Monitoring](api/monitoring.md) - Metrics and health checks
- [Performance](api/performance.md) - Performance optimization APIs

### 💡 Examples
- [Basic CRUD](examples/basic-crud.md) - Create, Read, Update, Delete operations
- [Advanced Queries](examples/advanced-queries.md) - Complex query patterns
- [Connection Pooling](examples/connection-pooling.md) - Using connection pools
- [Async Operations](examples/async-operations.md) - Asynchronous database operations
- [Web Application](examples/web-app.md) - Building a web application
- [Microservices](examples/microservices.md) - Microservice architecture patterns
- [Testing Strategies](examples/testing.md) - Testing approaches and patterns

### 🔍 Technical Reference
- [Architecture](reference/architecture.md) - Internal architecture overview
- [SQL Compatibility](reference/sql-compatibility.md) - GaussDB SQL feature support
- [Type Mappings](reference/type-mappings.md) - Complete type mapping reference
- [Performance Benchmarks](reference/benchmarks.md) - Performance characteristics
- [Migration Guide](reference/migration.md) - Migrating from other ORMs
- [Changelog](reference/changelog.md) - Version history and changes

## 🎯 Quick Navigation

### For Beginners
1. Start with the [Quick Start Guide](guides/getting-started.md)
2. Learn about [Configuration](guides/configuration.md)
3. Try the [Basic CRUD Example](examples/basic-crud.md)
4. Read [Best Practices](guides/best-practices.md)

### For Experienced Users
1. Check the [API Reference](api/) for detailed information
2. Explore [Advanced Examples](examples/advanced-queries.md)
3. Review [Performance Optimization](guides/performance.md)
4. See [Architecture Details](reference/architecture.md)

### For Contributors
1. Read the [Contributing Guide](../../CONTRIBUTING.md)
2. Check [Architecture Overview](reference/architecture.md)
3. Review [Testing Strategies](examples/testing.md)
4. See [Development Setup](guides/development.md)

## 🌟 Key Features

### Core Functionality
- **Full Diesel Compatibility**: 100% compatible with Diesel 2.2.x API
- **Type Safety**: Complete Rust type system integration
- **Query Builder**: Powerful and flexible query construction
- **Transactions**: Full transaction support with savepoints
- **Migrations**: Schema migration management

### Advanced Features
- **Connection Pooling**: R2D2 integration for connection management
- **Async Support**: Tokio runtime compatibility
- **Monitoring**: Built-in metrics and health checks
- **Performance**: Query caching and batch operations
- **Security**: SSL/TLS support and query validation

### Database Support
- **GaussDB**: Full support for GaussDB features
- **OpenGauss**: Compatible with OpenGauss databases
- **PostgreSQL**: Leverages PostgreSQL compatibility

## 🚀 Quick Example

```rust
use diesel::prelude::*;
use diesel_gaussdb::GaussDbConnection;

// Connect to database
let mut conn = GaussDbConnection::establish(&database_url)?;

// Query users
let users = users::table
    .select(User::as_select())
    .load(&mut conn)?;

// Insert new user
let new_user = NewUser {
    name: "John Doe",
    email: "john@example.com",
};

let user = diesel::insert_into(users::table)
    .values(&new_user)
    .returning(User::as_returning())
    .get_result(&mut conn)?;
```

## 📊 Supported Versions

- **diesel-gaussdb**: 1.0+
- **Diesel**: 2.2.x
- **Rust**: 1.70.0+ (MSRV)
- **GaussDB**: 505.2.0+
- **OpenGauss**: 7.0.0+

## 🤝 Community and Support

### Getting Help
- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues) - Bug reports and feature requests
- [GitHub Discussions](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/discussions) - Community discussions
- [Huawei Cloud Forum](https://bbs.huaweicloud.com/forum/forum-1131-1.html) - GaussDB technical support

### Contributing
- [Contributing Guide](../../CONTRIBUTING.md) - How to contribute
- [Code of Conduct](../../CODE_OF_CONDUCT.md) - Community guidelines
- [Development Setup](guides/development.md) - Setting up development environment

### Resources
- [Official Website](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel)
- [Crates.io](https://crates.io/crates/diesel-gaussdb)
- [Documentation](https://docs.rs/diesel-gaussdb)
- [Examples Repository](https://github.com/HuaweiCloudDeveloper/gaussdb-examples-rust)

## 📝 Documentation Standards

This documentation follows these principles:

- **Accuracy**: All examples are tested and working
- **Clarity**: Clear, concise explanations with practical examples
- **Completeness**: Comprehensive coverage of all features
- **Accessibility**: Suitable for both beginners and experts
- **Maintenance**: Regularly updated with new releases

## 🔄 Version Information

This documentation covers diesel-gaussdb version 1.0 and is compatible with:

- Diesel 2.2.x
- GaussDB 505.2.0+
- OpenGauss 7.0.0+
- Rust 1.70.0+ (MSRV)

For version-specific information, see the [Changelog](reference/changelog.md).

## 📞 Feedback

We value your feedback! If you have suggestions for improving this documentation:

1. [Open an issue](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/issues/new) with the `documentation` label
2. [Start a discussion](https://github.com/HuaweiCloudDeveloper/gaussdb-diesel/discussions) in our community forum
3. Submit a pull request with improvements

---

**Ready to build amazing applications with diesel-gaussdb? Let's get started!** 🚀

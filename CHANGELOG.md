# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha] - 2025-07-29

### Added
- Initial implementation of GaussDB backend for Diesel ORM
- Core `GaussDB` backend struct implementing `Backend` trait
- `GaussDBQueryBuilder` with PostgreSQL-compatible SQL generation
- `GaussDBValue` type for raw database values
- `GaussDBTypeMetadata` for type information management
- Support for basic SQL types:
  - Numeric types: `SmallInt`, `Integer`, `BigInt`, `Float`, `Double`
  - Text types: `Text`
  - Binary types: `Binary`
  - Date/Time types: `Date`, `Time`, `Timestamp`
  - Boolean type: `Bool`
- PostgreSQL-compatible SQL dialect implementation
- Parameter binding with PostgreSQL-style placeholders ($1, $2, ...)
- Identifier quoting and escaping
- Comprehensive test suite with 7 integration tests
- Basic usage example demonstrating query building
- Complete project documentation and README

### Technical Details
- Implements Diesel's `Backend`, `SqlDialect`, and `TypeMetadata` traits
- Uses `RawBytesBindCollector` for parameter binding
- Supports PostgreSQL-compatible OID type mapping
- Modular architecture with separate modules for backend, connection, query_builder, types
- Feature-based dependency management
- Full compatibility with Diesel 2.2+ ecosystem

### Development Infrastructure
- Cargo project structure with proper metadata
- MIT/Apache-2.0 dual licensing
- Comprehensive unit and integration tests
- Example code and documentation
- Development-ready project structure

### Known Limitations
- Connection implementation is placeholder (planned for phase 3)
- GaussDB-specific types not yet implemented
- Authentication mechanisms not yet implemented
- Async support not yet implemented

### Next Steps
- Phase 3: Implement actual database connection functionality
- Phase 4: Add GaussDB-specific data types and features
- Phase 5: Implement advanced query builder extensions

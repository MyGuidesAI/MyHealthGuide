# Repository Module Structure

This directory contains the modularized components of the repository layer, which was previously a single large file (`repository.rs`). The modularization improves code organization, maintainability, and testability.

## Structure

- **mod.rs**: Entry point that re-exports public components and defines the module structure
- **errors.rs**: Defines the `RepositoryError` type and error handling utilities
- **blood_pressure.rs**: Implements the `BloodPressureRepository` as the main API for blood pressure data
- **in_memory.rs**: Provides an in-memory storage implementation used for testing and as a fallback
- **storage.rs**: Contains database-specific implementations for different storage backends

## Design Pattern

The repository pattern is used to abstract data access:

1. `BloodPressureRepository` provides high-level methods for the application to use
2. Repository methods first try to use the configured database backend (`DatabaseStorage`)
3. If database access fails, it falls back to in-memory storage (`InMemoryStorage`)

This design provides:
- Resilience to database unavailability
- Easy extension for new database backends through feature flags
- Clean separation between data access and business logic

## Adding New Storage Backends

To add a new storage backend:

1. Add feature flag in `Cargo.toml`
2. Extend the `DatabasePool` enum in the database module
3. Add conditional compilation blocks in `storage.rs` for the new backend
4. Implement the appropriate database-specific operations

## Testing

The repository layer is designed to be easily testable:
- In-memory storage can be used for isolated unit tests
- Integration tests can verify database interactions
- Mock implementations of the repository can be created for testing higher-level components

## Background and Legacy Support

This modular structure replaced a single `repository.rs` file. The original file still exists but now simply re-exports from this module structure to maintain backward compatibility. 
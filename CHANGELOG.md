# Changelog

All notable changes to the MyHealthGuide project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup with three main crates:
  - MyHealthGuide-api: Public API layer
  - MyHealthGuide-domain: Business logic and domain services
  - MyHealthGuide-data: Data access and storage abstraction
- REST API endpoints for blood pressure tracking
- User authentication with OAuth2/OIDC support
- Health check endpoint (/health) for monitoring
- Docker containerization for development and production
- SQLite database support for local development
- Support for database migrations
- Basic documentation in the docs/ directory

### Changed
- N/A (initial release)

### Fixed
- N/A (initial release)

## [0.1.0] - YYYY-MM-DD
- Initial release (future) 
# MyHealthGuide

A complete health tracking platform built with Rust, focusing on scalability, performance, and security.

## Project Structure

The project is organized into three main crates:

- **MyHealthGuide-api**: Public API layer for the application
- **MyHealthGuide-domain**: Business logic and domain services
- **MyHealthGuide-data**: Data access and storage abstraction

## Features

- **Blood Pressure Tracking**: Record and analyze blood pressure readings
- **API Documentation**: Integrated Swagger UI for API exploration
- **Authentication**: Support for OAuth2 and OIDC
- **Health Checks**: Built-in health endpoints for monitoring
- **Database Abstraction**: Support for SQLite, MySQL, and PostgreSQL databases

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or higher
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose (for containerized deployment)

### Local Development

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/MyHealthGuide.git
   cd MyHealthGuide
   ```

2. Create a `.env` file from the example:
   ```
   cp .env.example .env
   ```

3. Build and run the application:
   ```
   cargo build
   cargo run
   ```

4. For development with automatic reloading:
   ```
   cargo watch -x run
   ```

### Docker Deployment

1. Build and start the container:
   ```
   docker compose up dev --build
   ```

2. For production deployment:
   ```
   docker compose up prod --build
   ```

## API Documentation

Once the application is running, visit:
- http://localhost:3000/swagger-ui/ for API documentation
- http://localhost:3000/health for system health status

## Project Structure

```
MyHealthGuide/
├── .github/                # GitHub workflows and templates
├── MyHealthGuide-api/      # API Layer
├── MyHealthGuide-domain/   # Domain Logic
├── MyHealthGuide-data/     # Data Access Layer
├── docs/                   # Documentation
│   ├── ROADMAP.md          # Development roadmap
│   ├── DOCKER_GUIDE.md     # Docker usage guide
│   ├── README-AUTH0.md     # Auth0 integration guide
│   └── ...                 # Other documentation
├── scripts/                # Utility scripts
└── tests/                  # Integration tests
```

## Development Tools

This project uses several tools to maintain code quality:

- **GitHub Actions**: Automated CI/CD pipeline for testing and building
- **Clippy**: Static code analysis to catch common mistakes
- **Rustfmt**: Code formatting to maintain consistent style
- **EditorConfig**: Consistent coding style across different editors
- **Docker**: Containerization for consistent deployment environments

## Documentation

For more detailed documentation, see the `/docs` directory:

- [Development Roadmap](docs/ROADMAP.md)
- [Docker Guide](docs/DOCKER_GUIDE.md)
- [Auth0 Integration](docs/README-AUTH0.md)
- [Auth Logging](docs/AUTH-LOGGING-README.md)
- [OIDC Troubleshooting](docs/OIDC-TROUBLESHOOTING.md)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes in each release.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 

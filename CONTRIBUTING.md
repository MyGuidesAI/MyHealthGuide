# Contributing to MyHealthGuide

Thank you for considering contributing to MyHealthGuide! This document outlines the process for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/MyHealthGuide.git`
3. Create a new branch for your feature or bug fix: `git checkout -b feature/your-feature-name`

## Development Setup

1. Install Rust and Cargo using [rustup](https://rustup.rs/)
2. Copy `.env.example` to `.env` and adjust settings as needed
3. Run `cargo build` to build the project
4. Run `cargo test` to ensure all tests pass

## Project Structure

The project is organized into three main crates:

- **MyHealthGuide-api**: Public API layer for the application
- **MyHealthGuide-domain**: Business logic and domain services
- **MyHealthGuide-data**: Data access and storage abstraction

## Coding Standards

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format your code
- Ensure your code passes `cargo clippy` without warnings
- Write unit tests for new functionality
- Update documentation as needed

## Pull Request Process

1. Ensure your code builds and passes all tests
2. Update the README.md if necessary
3. Make sure your commit messages are clear and descriptive
4. Push your changes to your fork
5. Submit a pull request to the main repository

## Commit Message Guidelines

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Example:
```
Add blood glucose tracking feature

- Add database migrations for glucose readings table
- Implement CRUD operations for glucose readings
- Add API endpoints and validation
- Update documentation

Fixes #123
```

## Code Review Process

All submissions require review. We use GitHub pull requests for this purpose.

## Adding New Dependencies

Before adding a new dependency, consider:
- Is it actively maintained?
- Is it widely used in the Rust community?
- Does it have a compatible license?
- Could we implement the functionality ourselves without much effort?

## Testing

- Write unit tests for all new functionality
- Include integration tests for API endpoints
- Run the full test suite before submitting a pull request

## Documentation

- Update the README.md for user-facing changes
- Add inline documentation for public APIs
- Consider adding examples to the docs directory

## License

By contributing, you agree that your contributions will be licensed under the project's MIT License.

Thank you for contributing to MyHealthGuide! 
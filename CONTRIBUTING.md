# Contributing to VEDA

Thank you for your interest in contributing to VEDA! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please be respectful and constructive in all interactions.

## How to Contribute

### Reporting Bugs

When reporting bugs, please include:
- VEDA version
- Operating system and architecture
- Minimal reproduction case
- Expected vs actual behavior
- Any error messages or panics

### Suggesting Features

Feature requests should include:
- Use case description
- Proposed API design
- Performance considerations
- Compatibility impact

### Pull Requests

1. **Fork and Branch**: Create a feature branch from `main`
2. **Write Tests**: All new features must include tests
3. **Document**: Add documentation for public APIs
4. **Benchmark**: Include benchmarks for performance-critical code
5. **Format**: Run `cargo fmt` before committing
6. **Lint**: Ensure `cargo clippy` passes
7. **PR**: Submit pull request with clear description

## Development Setup

```bash
# Clone the repository
git clone https://github.com/veda-rs/veda
cd veda

# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-features -- -D warnings
```

## Code Style

- Follow Rust standard style (enforced by rustfmt)
- Use meaningful variable and function names
- Prefer explicitness over brevity
- Add comments for complex algorithms
- Document all public APIs

## Testing

- Write unit tests for individual functions
- Write integration tests for feature combinations
- Add stress tests for concurrency issues
- Ensure tests are deterministic and don't flake

## Performance

- Benchmark performance-critical code
- Avoid allocations in hot paths
- Profile before optimizing
- Document performance characteristics

## Documentation

- All public items must have documentation
- Examples in documentation must compile and run
- Update README.md for user-facing changes
- Add examples for new features

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.

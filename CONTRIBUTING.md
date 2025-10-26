# Contributing

Thanks for considering contributing to VEDA.

## Code of Conduct

Be respectful and constructive.

## How to Contribute

### Reporting Bugs

Include:
- Version
- OS and architecture
- Minimal repro
- Expected vs actual behavior
- Error messages

### Suggesting Features

Include:
- Use case
- Proposed API
- Performance impact

### Pull Requests

1. Fork and create a branch from `main`
2. Write tests
3. Document public APIs
4. Run `cargo fmt`
5. Make sure `cargo clippy` passes
6. Submit PR

## Development Setup

```bash
git clone https://github.com/veda-rs/veda
cd veda
cargo build
cargo test
cargo bench
cargo fmt --check
cargo clippy --all-features -- -D warnings
```

## Code Style

- Follow rustfmt
- Use meaningful names
- Comment complex code
- Document public APIs

## Testing

- Write unit tests
- Write integration tests
- Add stress tests for concurrency
- Make tests deterministic

## Performance

- Benchmark perf-critical code
- Avoid allocations in hot paths
- Profile first
- Document perf characteristics

## Documentation

- Document public items
- Make sure doc examples compile
- Update README for user-facing changes
- Add examples

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.

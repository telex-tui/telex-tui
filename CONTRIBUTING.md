# Contributing to Telex

Thanks for your interest in contributing to Telex!

## Project Governance

Telex is maintained by **Mark Branion**, who retains final decision-making authority over:
- Project direction and roadmap
- API design and breaking changes
- Which contributions are accepted
- Release timing and versioning

This is a **benevolent dictatorship** model - contributions are welcome, but the maintainer has the final say on what gets merged.

## How to Contribute

### Reporting Issues

- Check existing issues first to avoid duplicates
- Provide clear reproduction steps
- Include your terminal emulator, OS, and Rust version
- For rendering bugs, include screenshots if possible

### Proposing Changes

Before investing significant time:

1. **Open an issue first** to discuss the change
2. Wait for maintainer feedback on whether it aligns with project goals
3. Smaller, focused PRs are more likely to be accepted than large rewrites

### Pull Requests

- Fork the repository and create a feature branch
- Follow existing code style and conventions
- Add tests for new functionality (see `docs/testing.md`)
- Run `cargo test` before submitting
- Keep PRs focused on a single concern
- Be prepared for requested changes or rejection

### What Gets Accepted

Priority is given to:
- Bug fixes with tests
- Performance improvements
- Documentation improvements
- Examples demonstrating framework features

Lower priority:
- New widgets (framework is still stabilizing)
- Large architectural changes (unlikely to be accepted)
- Breaking API changes (requires strong justification)

## Code of Conduct

- Be respectful and professional
- Accept constructive feedback gracefully
- Focus on technical merit, not personal preferences
- The maintainer's decision is final

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

All contributions must be your own work or properly attributed, and you must have the right to submit them under the MIT License.

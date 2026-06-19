# Contributing to cf-ddns

Thank you for your interest in contributing to `cf-ddns`!

To keep the codebase clean, stable, and consistent, please follow these guidelines when contributing.

## Development Workflow

Before submitting a pull request, verify that your changes pass formatting, compilation, lints, and tests locally:

1. **Build**:

   ```bash
   cargo build
   ```

2. **Lint & Format Check**:

   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Tests**:

   ```bash
   cargo test
   ```

## Commit Message Convention

We use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for structuring commit messages. This allows automatic changelog generation and easier tracking of version increases.

Commit messages should follow the structure:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Common types:

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Changes that do not affect the meaning of the code (formatting, missing semi-colons, etc.)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

## Changelog Convention

`CHANGELOG.md` follows the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format and adheres to [Semantic Versioning](https://semver.org/).

- **User-Facing Changes Only**: Only document changes that have an observable effect on the user (e.g. new features, behavior changes, bug fixes, deprecations, removals, and security fixes).
- **Exclude Internal Refactors**: Do not add internal refactors, code quality improvements, or dependency updates with no user-observable impact to the changelog.

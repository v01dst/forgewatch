# Contributing

Thanks for improving ForgeWatch. Please keep changes focused, typed, tested, and honest about analysis limitations.

## Workflow

1. Discuss significant analyzer behavior or JSON changes in an issue first.
2. Create a feature branch.
3. Add or update tests for analyzer behavior and CLI contracts.
4. Run the complete validation suite.
5. Submit a clear pull request describing behavior and rationale.

## Validation

All submissions must pass:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Guidelines

- Prefer small modules and explicit data structures.
- Never panic while analyzing user files.
- Preserve schema version compatibility unless the change is intentionally coordinated.
- Label heuristic results as partial rather than implying deep compiler-level analysis.
- Bound file reads and avoid loading arbitrarily large repositories into memory.
- Keep terminal output readable without color.

# Analysis Rules

ForgeWatch intentionally favors transparent heuristics over opaque claims.

## File scanning

The scanner respects project ignore rules and avoids following symlink cycles. Binary files are excluded from source metrics using a lightweight NUL-byte probe.

## Source metrics

Metrics such as line count, comment ratio, TODO/FIXME count, long lines, and oversized files are signals rather than proof of code quality. They should be reported with enough context for users to interpret them.

## Dependencies

Manifest parsing is intentionally conservative. Unsupported manifests are ignored rather than guessed at, and registry access is not required for a normal scan.

## Git state

Git information describes the repository at scan time: branch, status, ahead/behind information, author, timestamp, and the latest commit.

## Health scoring

Scores are derived from explicit issue penalties. When adding a new penalty, document the observed signal, why it matters, the penalty range, how the signal is tested, and whether it can be unavailable.

A missing signal should not silently become a false negative.

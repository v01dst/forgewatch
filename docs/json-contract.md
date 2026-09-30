# JSON Contract

The JSON renderer is intended for scripts and CI integrations.

## Stability

The current schema version is 1. Additive fields may be introduced without changing the version. Existing field names and meanings should remain stable within version 1.

## Errors

Errors are emitted as a structured object on stderr and the process exits non-zero:

    {"error":{"message":"..."}}

Consumers should not parse human terminal output when JSON mode is available.

## Partial analysis

The partial_analysis field records conditions that limited inspection. Examples include unreadable files, unsupported manifests, missing Git metadata, or platform-specific limitations.

## Compatibility

Changes to the JSON contract should include fixture updates and tests that verify both newly added fields and the preservation of existing fields.

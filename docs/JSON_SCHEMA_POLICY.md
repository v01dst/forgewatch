# JSON schema policy

ForgeWatch exposes JSON for scripts and automation, so output stability is part of the CLI contract.

## Compatibility rules

- Keep the top-level schema_version field.
- Add new fields without changing the meaning of existing fields.
- Do not rename or remove existing fields within schema version 1.
- Keep machine-readable errors on stderr with a non-zero exit code.
- Keep human-readable terminal output separate from JSON output.

## Adding fields

When adding a field:
1. define its meaning and whether it can be absent;
2. update the JSON renderer;
3. add or update a fixture;
4. add a regression test;
5. document the field in the README example when it affects consumers.

## Breaking changes

Renaming, removing a field, or changing its type requires a schema-version decision. Prefer a new additive field and a deprecation period when practical.

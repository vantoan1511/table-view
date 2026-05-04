# Oracle Driver Temporary Solution

## Status

Oracle support in `extensions/db-bridge` currently uses a temporary vendored dependency:

- `extensions/db-bridge/vendor/oracle-rs`

This is intentional for now.

## Why This Exists

The app uses `oracle-rs` as a pure Rust thin driver so Oracle support does not depend on Oracle Instant Client, OCI, or ODPI-C.

At the moment, we need a small local patch to `oracle-rs` for the authentication flow used by privileged Oracle roles such as:

- `SYSDBA`
- `SYSOPER`

Because of that patch, `extensions/db-bridge/Cargo.toml` overrides crates.io with:

```toml
[patch.crates-io]
oracle-rs = { path = "vendor/oracle-rs" }
```

Without this vendored source, the current Oracle login flow would lose the local auth-mode fix.

## Packaging Behavior

This vendored source is only needed to build the extension.

It is already excluded from production packaging in `neutralino.config.json`:

- `cli.extensionsExclude` contains `vendor`
- source files such as `.rs` and `Cargo.*` are also excluded

That means shipped builds should still contain only the runtime extension binary, not the Rust source tree used during development.

## Target End State

The intended long-term structure is:

- `extensions/` contains only built extension binaries and runtime assets
- Rust source for `db-bridge` and patched dependencies live in a separate repository

## Planned Migration

When we split the extension into its own repository, we should:

1. Move `extensions/db-bridge` Rust source into a dedicated repo.
2. Move the patched `oracle-rs` source there as well, or replace it with:
   - an upstream release that includes the needed auth support, or
   - a Git dependency pointing to our maintained fork.
3. Build `db-bridge.exe` in CI from that separate repo.
4. Publish or copy only the built binary into this app repo under `extensions/db-bridge/`.
5. Remove build-only Rust sources from this repo once the external build pipeline is stable.

## Notes

- Keep the vendored `oracle-rs` folder until the separate extension repo exists or upstream/fork-based dependency management replaces it.
- If Oracle auth behavior is changed later, check both:
  - `extensions/db-bridge/src/drivers/oracle.rs`
  - `extensions/db-bridge/vendor/oracle-rs`

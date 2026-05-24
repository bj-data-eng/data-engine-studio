# Security Policy

Data Engine Studio is early-stage software. This file documents the repository security checks that are currently in place and the commands maintainers should run before merging dependency, build, packaging, or vendored-code changes.

## Reporting Security Issues

Please do not disclose suspected vulnerabilities publicly before maintainers have had a chance to investigate. If GitHub private vulnerability reporting is enabled for this repository, use that. Otherwise, contact the repository maintainers through the least-public channel available to you and include:

- the affected component or dependency,
- the impact you believe is possible,
- reproduction steps or a proof of concept when safe to share,
- any relevant platform details.

## Dependency And Supply Chain Policy

Rust dependencies are locked in `Cargo.lock` and direct dependency requirements are pinned exactly in the workspace manifests. Internal path dependencies also carry exact workspace versions so policy checks can reject wildcard dependency declarations.

Python packaging currently has no runtime third-party dependencies. The Python build backend is pinned exactly in `pyproject.toml`.

Vendored or forked code must keep its provenance visible in the crate metadata and notices. The current tree includes:

- `crates/des-document/layout`, a vendored layout engine derived from Taffy.
- `crates/des-graph-egui`, a vendored graph interaction crate that was audited before promotion into the workspace.
- `vendor/des-apple-dispatch`, a small local replacement for the external `dispatch` crate, patched through `[patch.crates-io]`.

## Required Security Checks

Run the full security target:

```sh
just security
```

This currently runs:

```sh
cargo audit
cargo deny check
```

`cargo audit` scans `Cargo.lock` against the RustSec advisory database.

`cargo deny check` enforces the repository policy in `deny.toml`:

- no ignored advisories by default,
- only approved license families,
- no wildcard dependency requirements,
- unknown registries are denied,
- unknown Git dependencies are denied,
- only the crates.io index is allowed as a registry source,
- no Git sources are allowed.

## Related Verification

Security-sensitive dependency or build changes should also run the normal verification path:

```sh
just verify
```

For environments without `cargo nextest`, run:

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
```

Python packaging changes should also run:

```sh
just python-test
just python-smoke
```

Use `just verify-all` before security-sensitive merges when the local environment
can build the native Python extension.

## When To Re-Audit

Re-run and review the checks above whenever a change:

- adds, removes, or updates a dependency,
- changes `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, or `deny.toml`,
- changes vendored or forked code,
- adds `unsafe` Rust,
- adds filesystem, process, network, dynamic loading, or template/file reload behavior,
- changes packaging, build scripts, release scripts, or launcher behavior.

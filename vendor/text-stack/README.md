Text Stack Vendor Sources
=========================

This directory contains the full external dependency closure currently used by
`crates/des-ui-text`.

These crates were copied from the local Cargo registry and are patched in
`Cargo.toml` through `[patch.crates-io]`, so the text engine builds from
repository-owned source instead of fetching this stack from crates.io.

Keep upstream license and attribution files in each vendored crate. Changes made
inside this tree are now maintained as project code.

# block 0.1.6 provenance

Vendored from the locally cached crates.io archive `block-0.1.6.crate` on 11 October 2026.

- Registry: `https://github.com/rust-lang/crates.io-index`
- Archive SHA-256: `0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a` (matches the original workspace lockfile checksum).
- Upstream: https://github.com/SSheldon/rust-block
- Upstream tag `0.1.6` resolves to `47178790cfc9d4a8b092051d8b413b78bd31254a`, read through the GitHub API in this run. Source files were taken from the registry archive, not the current branch.
- Author metadata: Steven Sheldon.
- License metadata: MIT. The original Cargo.toml, README.md and test utility source are retained verbatim.
- The published archive contains no separate license notice; the inspected upstream root likewise lists no license file. No copyright year or upstream license text has been invented. Licensing evidence is the preserved package declaration.

## Local change

In `src/lib.rs`, replace private `enum Class { }` with `#[repr(C)] struct Class { _private: [u8; 0] }` and an address-only comment. The external symbol declaration, pointer field, runtime functions, public API and all other source remain unchanged. Package version remains 0.1.6. The workspace selects this source using `[patch.crates-io]`.

The original manifest references an unpublished local `test_utils` directory as a development dependency; that directory is absent from the published archive. No development dependency was changed or downloaded. Direct rustdoc tests can run against a directly compiled library without resolving that missing helper.

# block 0.1.6 compatibility validation

T-0069 · 11 October 2026 · branch `task/T-0069-block-compatibility`

## Result

The vendored, version-preserving replacement removes `uninhabited_static` on aarch64 macOS with Rust 1.99.0. Both inverse edges resolve to the same local block 0.1.6. Fresh locked build, workspace tests and Clippy pass. Application source and GPU package versions are unchanged.

The only production source substitution is private `enum Class { }` → `#[repr(C)] struct Class { _private: [u8; 0] }`, with an address-only comment. Cargo.toml selects the local crate at the workspace root. Cargo.lock removes only block's registry source and checksum, retaining version 0.1.6. An initial targeted Cargo update also pruned the stale phosphlux-lite entry; that unrelated pruning was restored before the final locked build. Archive provenance, author and original MIT metadata are preserved in `vendor/block-0.1.6/PROVENANCE.md` and the unchanged dependency manifest. The archive and inspected upstream root supply no separate license file.

## ABI review

Source inspection finds Class only at its declaration, the external `_NSConcreteStackBlock` declaration, and the `*const Class` field in the repr(C) block header. Construction takes the symbol address. Class is never allocated, passed by value, dereferenced for data, or used as storage. The replacement is inhabited, zero sized, and alignment 1; the header still contains a thin pointer at the same position. No foreign function signature, symbol name, calling convention, or block header field changes.

The [Clang Block ABI](https://clang.llvm.org/docs/Block-ABI-Apple.html) specifies a pointer in the isa slot and initialization to the runtime symbol address. The [Rust issue](https://github.com/rust-lang/rust/issues/74840) concerns the compiler assumptions violated by an uninhabited external object. Together with the observed uses, this supports the replacement for an address-only opaque symbol. This is a source-based ABI assessment, supplemented by macOS runtime checks below; it is not certification for every supported target.

## Commands and observed output

All commands ran in this session from the repository root. Cargo validation used `CARGO_TARGET_DIR=/tmp/phosphlux-check/T-0069-target` and `CARGO_TERM_COLOR=never`. The directory was fresh for the first build. Logs and the temporary smoke harness are under `/tmp/phosphlux-check/T-0069`.

| Command | Exit | Observed output |
| --- | --- | --- |
| `rustc -vV` | 0 | `rustc 1.99.0 (b940084d7 2026-09-28)`, host `aarch64-apple-darwin`, LLVM 23.1.1 |
| `cargo -V` | 0 | `cargo 1.99.0 (5f94df478 2026-08-27)` |
| `cargo update --offline -p block@0.1.6` | 0 | `Adding block v0.1.6` from the local vendor path; registry block removed |
| `cargo tree --locked --offline -i block@0.1.6` | 0 | Local block 0.1.6 has direct inverse edges from metal 0.29.0 and wgpu-hal 23.0.1; full output in tree.log |
| `cargo build --locked --offline --future-incompat-report` | 0 | Fresh build: `Finished ... in 27.05s`; block generated 46 warnings; phosphlux generated 66 warnings |
| `cargo test --locked --offline` | 0 | `test result: ok. 8 passed; 0 failed`; phosphlux test target generated 60 warnings |
| `cargo clippy --locked --offline` | 0 | `Finished ... in 5.95s`; phosphlux generated 106 warnings |
| `cargo report future-incompatibilities` | 101 | `error: no reports are currently available` in the fresh target directory |
| Final `cargo build --locked --offline --future-incompat-report` after restoring the unrelated lock entry | 0 | `Finished ... in 0.14s`; `note: 0 dependencies had future-incompatible warnings` |

Direct crate checks (no Cargo lint cap):

- `rustc --crate-name block --crate-type rlib --deny uninhabited_static vendor/block-0.1.6/src/lib.rs -o /tmp/phosphlux-check/T-0069/libblock.rlib` — exit 0, 46 existing missing-ABI warnings; no uninhabited-static diagnostic.
- Initial `rustdoc --test vendor/block-0.1.6/src/lib.rs --extern block=/tmp/phosphlux-check/T-0069/libblock.rlib` — exit 101, both tests failed to import block because the direct invocation omitted its crate name.
- Corrected `rustdoc --test --crate-name block vendor/block-0.1.6/src/lib.rs --extern block=/tmp/phosphlux-check/T-0069/libblock.rlib` — exit 0: `2 passed; 0 failed`. One example exercises block creation, heap copy and invocation; the other compiles an invocation function.
- `rustc /tmp/phosphlux-check/T-0069/block-smoke.rs --extern block=/tmp/phosphlux-check/T-0069/libblock.rlib -o /tmp/phosphlux-check/T-0069/block-smoke` — exit 0. Running that binary — exit 0: `PASS: stack isa address, invoke, heap copy, clone, final captured-value drop`. This independently checks the actual header's runtime symbol address and captured-value lifetime; no Phosphlux app was launched.

A final Python assertion audit exited 0: archive Cargo.toml, README and test utility source are byte-identical; lib.rs differs by exactly the Class substitution and comment; Cargo.lock differs only by the block source/checksum removal. `git diff --exit-code -- apps tools` exited 0. Log scans found no `static of uninhabited type` diagnostic in build, test, Clippy, final-build or direct rustc logs. The archive SHA-256 command exited 0 and matched the original lock checksum.

## Remaining warnings and limits

Moving block from a registry dependency to local source exposes 46 existing `missing_abi` warnings (implicit extern C declarations); some printed notes mention future rejection, but Cargo's future-incompatibility report is empty. These were not suppressed or rewritten. Existing application warnings remain. This task removes the diagnosed uninhabited static, not every warning in block or Phosphlux.

The crate's six original unit tests were not run: they depend on objc_test_utils, whose local directory is absent from the published archive and whose crate is not cached. No dependency was installed to fill that gap. Its documentation examples and the independent smoke check ran instead. No app launch, GPU rendering test, release build, iOS or other target/toolchain check, push, merge, or state-file edit occurred. ABI preservation outside the tested macOS host remains a source-supported assessment. The missing separate upstream license notice remains a provenance limitation; the original MIT declaration is preserved.

Existing untracked BASELINE.md and research/schematics were present at session start and left untouched. No additional implementation task is established by these checks; external verification and creator acceptance remain board steps.

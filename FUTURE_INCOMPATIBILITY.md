# block future incompatibility

T-0068 · 11 October 2026

## Finding

The locked macOS build succeeds, but `block 0.1.6` declares an external static with an uninhabited type. Its private `enum Class { }` has no variants. The static is used by address in `ConcreteBlock::with_invoke` (`isa: &_NSConcreteStackBlock`, line 261), so the declaration is relevant to block construction.

The installed Rust 1.99.0 already sets `uninhabited_static` to **deny** by default. A successful Cargo dependency build does not establish that this declaration is acceptable without lint capping. The exact release that first rejects it without a cap was not established.

## Reproduction

Commands ran from the repository root. No installs, dependency updates, app launch, pushes or merges were performed.

`rustc -vV` and `cargo -V` — exit 0:

```text
rustc 1.99.0 (b940084d7 2026-09-28)
host: aarch64-apple-darwin
LLVM version: 23.1.1
cargo 1.99.0 (5f94df478 2026-08-27)
```

`CARGO_TARGET_DIR=/tmp/phosphlux-check/target CARGO_TERM_COLOR=never cargo build --locked --offline --future-incompat-report` — exit 0, cached build (0.14s). Log: `/tmp/phosphlux-check/T-0068-build.log`.

To exclude stale dependency diagnostics, a second build used a new target directory:

```sh
CARGO_TARGET_DIR=/tmp/phosphlux-check/T-0068-target CARGO_TERM_COLOR=never cargo build --locked --offline --future-incompat-report
```

Exit 0. Log: `/tmp/phosphlux-check/T-0068-fresh-build.log`. Output:

```text
warning: `phosphlux` (bin "phosphlux") generated 66 warnings (run `cargo fix --bin "phosphlux" -p phosphlux` to apply 26 suggestions)
    Finished `dev` profile [optimized + debuginfo] target(s) in 27.21s
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
```

`CARGO_TARGET_DIR=/tmp/phosphlux-check/T-0068-target CARGO_TERM_COLOR=never cargo report future-incompatibilities --id 1 --package block@0.1.6` — exit 0. Log: `/tmp/phosphlux-check/T-0068-report.log`. Exact diagnostic (Cargo's quotation prefixes retained):

```text
> warning: static of uninhabited type
>   --> /Users/donovan/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/block-0.1.6/src/lib.rs:64:5
>    |
> 64 |     static _NSConcreteStackBlock: Class;
>    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
>    |
>    = note: uninhabited statics cannot be initialized, and any access would be an immediate error
>    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
>    = note: for more information, see issue #74840 <https://github.com/rust-lang/rust/issues/74840>
```

## Locked dependency path

`cargo tree --locked --offline -i block@0.1.6` — exit 0:

```text
block v0.1.6
├── metal v0.29.0
│   └── wgpu-hal v23.0.1
│       ├── wgpu v23.0.1
│       │   ├── egui-wgpu v0.30.0
│       │   │   └── phosphlux v0.1.0
│       │   └── phosphlux v0.1.0
│       └── wgpu-core v23.0.1
│           └── wgpu v23.0.1 (*)
└── wgpu-hal v23.0.1 (*)
```

The tree above omits only the absolute Phosphlux package directory. Both direct and `metal` paths matter: changing only `metal` would not remove the direct `wgpu-hal → block` edge.

Read-only inspection of cached `wgpu-hal-23.0.1/Cargo.toml` showed that its `metal` feature enables `dep:block`; the `block` and `metal` dependencies are under `cfg(any(target_os="macos", target_os="ios"))`. The observed warning is from an aarch64 macOS Metal build. Other targets were not built. `cargo tree --locked --offline --target x86_64-unknown-linux-gnu -i block@0.1.6` exited 101 because `arrayref v0.3.9` was not cached; no download was attempted beyond Cargo's rejected offline request.

## Toolchain condition

`rustc -W help | rg 'uninhabited.static|future.incompatible'` — exit 0, includes:

```text
uninhabited-static  deny     uninhabited static
```

The [rustc lint reference](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html#uninhabited-static) explains that an external declaration can bypass initialization checks for an impossible type, violating compiler assumptions. The [lint-level documentation](https://doc.rust-lang.org/rustc/lints/levels.html#capping-lints) explains that Cargo caps dependency lints with `--cap-lints allow`. This supports the interpretation of the successful dependency build despite the installed compiler's deny default; individual rustc invocation flags were not captured. A future unconditional compiler error would no longer be protected by a lint cap. No date or version for that transition is asserted.

## Smallest follow-up

Validate a local, version-preserving patch of `block 0.1.6` that replaces its private empty enum with an inhabited opaque zero-sized struct, then select that copy through a workspace-root `[patch.crates-io]` entry. This targets the reported declaration without requiring a wgpu/egui migration. The [Rust issue discussion](https://github.com/rust-lang/rust/issues/74840) recommends an inhabited zero-sized type for an address-only symbol; the cached source shows `Class` used as a raw pointer field and as the external symbol's type. The [Cargo override documentation](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section) supports selecting a patched dependency from the workspace root.

This is a source-supported candidate, not a tested fix. The follow-up must inspect ABI implications and licensing, check that both dependency edges select the patch, and run a fresh locked build and future-incompatibility report. The upstream [master source](https://github.com/SSheldon/rust-block/blob/master/src/lib.rs) inspected this run still contains the empty enum and static; no fixed published release was established. Changing application source, broadly upgrading the GPU stack, or merely hiding the warning is not supported as the smallest remedy by this diagnosis.

## Verification and limits

The fresh build and detailed report reproduced the declaration; cached dependency source confirmed the empty enum and its uses; the inverse tree confirmed both dependency edges. `git diff --exit-code` before adding this report exited 0, including an explicit check of Cargo.lock and both workspace manifests. The task branch is `task/T-0068-block-future-incompatibility`, based on `00c751a`.

Only this report is added. Existing untracked `BASELINE.md` and `research/schematics/` remain outside the task change. Tests, Clippy, runtime behavior, alternative toolchains, and the candidate patch were not exercised. Existing application warnings remain separate from this dependency diagnostic.

# Develop, test, and publish SimplyPDF

## Everyday edits

1. Reproduce the requested behavior with the current app or the closest existing test. Use a
   synthetic PDF for anything that might open, render, or modify a document.
2. Keep the existing Cargo target directory. Use the development build for day-to-day work so
   repeated edits reuse compiled dependencies and incremental output. Run Cargo commands serially
   in that directory so they can reuse the cache without waiting on each other’s build lock.
3. Run a focused regression first, then use `cargo xtask quick` with the affected package names.
   It runs formatting, asset and layer checks, then strict Clippy and tests for the named packages
   with two build jobs. For a documentation-only edit, omit the packages.
4. Verify the entry point the user will use. For desktop changes, launch the app with its opt-in
   `--control` channel and use `simplypdf-cli ui` to inspect the actual window. For automation
   changes, run the affected tools through the CLI and MCP where applicable. Check WASM when the
   change affects the web build. Run the repository’s release and parity checks as applicable.
5. Record the passing evidence, fetch `simplypdf/main`, and push one verified checkpoint. Use the
   normal GitHub CI checks to cover Linux and Windows. An edit can reach `main` independently of
   a new public download.

```sh
# Desktop or interface changes
cargo run --locked -p simplypdf -j 2 -- drawing.pdf
cargo xtask quick simplypdf-ui-egui

# CLI or automation changes
cargo xtask quick simplypdf-cli simplypdf-automation

# Documentation only
cargo xtask quick
```

## Publish a release

1. Run `cargo xtask smoke` from the release candidate. It runs the actual CLI against a synthetic
   PDF in a temporary directory, checks the persisted 6 m measurement, exports XLSX, and renders the
   saved page. For desktop changes, run the native UI smoke test too. Check that CI is green for
   that commit and inspect failed jobs at the failing step before retrying.
2. Pick a version and dispatch the existing [Release workflow](https://github.com/pchrisoc/simplypdf/actions/workflows/release.yml)
   once. Confirm the run’s commit SHA matches the tested source. Continue watching an active run
   for the same commit and version. Retry failed jobs on that run when possible so successful
   packages can be reused.
3. Review the draft. Verify its commit, platform file names, architecture, checksums, and signing
   status. GitHub generates the ZIP and tar.gz source archives from the published tag. Record which
   packages were actually installed or smoke-tested, along with any signing or coverage gaps.
4. Publish the draft when its downloads match the requested scope. Reopen the public page and
   verify the tag SHA and download list before reporting completion.

Use a local build for a quick preview. Full release builds optimize and package the app independently
for each platform, and should be reserved for candidates that are going to be distributed.

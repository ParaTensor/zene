# Release Guide

This guide describes how to release a new version of Zene to GitHub Releases and crates.io.

## 1. Preparation

Ensure your code is clean and tested:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## 2. Update Version

Edit `Cargo.toml` and update the version number:

```toml
[package]
version = "0.1.0"  # Update this
```

Commit the change:

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.0"
```

## 3. Publish to GitHub Releases

We use GitHub Actions to automatically build and release binaries.

1.  **Tag the commit**:
    ```bash
    git tag v0.1.0
    ```

2.  **Push the tag**:
    ```bash
    git push origin v0.1.0
    ```

The `.github/workflows/release.yml` workflow will automatically trigger, build the project for Linux, macOS, and Windows, and create a Draft Release on GitHub with the binaries attached.

## 4. Publish to crates.io

To publish to the Rust community registry:

1.  **Login** (if you haven't already):
    ```bash
    cargo login <your-api-token>
    ```
    *You can get a token at [crates.io/me](https://crates.io/me).*

2.  **Publish**:
    ```bash
    cargo publish
    ```

    *Note: `cargo publish` will automatically verify your package before uploading.*

## 5. Verify

*   Check [GitHub Releases](https://github.com/lipish/zene/releases) to ensure binaries are available.
*   Check [crates.io/crates/zene](https://crates.io/crates/zene) to ensure the new version is listed.

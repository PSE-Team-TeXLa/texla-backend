# TeXLa Backend

This is the backend of the graphical LaTeX editor TeXLa.

## Compilation

To compile a working binary from source follow these steps:

1. Build the [frontend](https://git.scc.kit.edu/pse-sose-2023-latex-team-2/frontend) as a production staticically
   hosted SPA.
2. Paste the build output into the `frontend` directory.
3. Build the project:
   ```shell
   cargo build --release
   ```
4. The binary and all the files it depends on can than be found in [`target/release`](./target/release)

You can then execute TeXLa by calling `target/release/backend.exe`.

## Bundling

Theoretically it is possible to bundle TeXLa into a distributable package using `cargo-bundle`.
However currently only `.deb` and `.app` (MacOS) packages are supported.

To bundle TeXLa follow these steps:

1. Install `cargo-bundle` using
   ```shell
   cargo install cargo-bundle
   ```
2. Run
   ```shell
   cargo bundle --release
   ```

The bundle is then placed in a subdirectory `deb` or `app` of [`target/release/bundle`](./target/release/bundle).

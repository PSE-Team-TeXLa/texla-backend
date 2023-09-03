# TeXLa Backend

This is the backend of the graphical LaTeX editor TeXLa.

## Compilation

To compile a working binary from source, follow these steps:

1. Build the [frontend](https://git.scc.kit.edu/pse-sose-2023-latex-team-2/frontend) as a statically hosted
   SPA (single-page application) for production.
   Step-by-step instructions can be found in the README of the frontend repository.
2. Paste the build output, i.e. the contents of the directory `build` in the frontend, into the directory `frontend` in
   the backend.
   For example, there should be a folder `/frontend/js`.
3. Build the project:
   ```shell
   cargo build --release
   ```
   When this command is run for the first time, all dependencies from [crates.io](https://crates.io) will be downloaded
   and compiled.
   This may take a while, but will not be necessary for subsequent builds.
4. The binary and all the files it depends on can then be found in [`target/release`](./target/release).
   You can now execute TeXLa by calling `target/release/texla`(followed by `.exe` on Windows).
5. To be able to run TeXLa as intended, i.e. by calling only `texla`, this binary has to be put into `PATH`.
   This can be achieved by running this in Windows PowerShell:
   ```shell
   $env:Path += ";" + (Resolve-Path target/release).Path
   ```
   or this in Unix Bash:
   ```shell
   export PATH=$PATH:$(pwd)/target/release
   ```
6. Now you can run TeXLa globally by typing `texla` in the command line.
   The CLI will provide you with usage information.

## Testing

To run all tests, execute:

```shell
cargo test --workspace
```

## Bundling

Theoretically, it is possible to bundle TeXLa into a distributable package using `cargo-bundle`.
However, currently only `.deb` (Debian) and `.app` (macOS) packages are supported.

To bundle TeXLa, follow these steps:

1. Install `cargo-bundle` using:
   ```shell
   cargo install cargo-bundle
   ```
2. Bundle up using:
   ```shell
   cargo bundle --release
   ```

The bundle is then placed in a subdirectory `deb` or `app` of [`target/release/bundle`](./target/release/bundle).

# Contributing

Workers-rs is an open source project and we welcome contributions from you. Thank you!

Below you can find some guidance on how to get started in contributing to the project.

## Getting started

### Set up your environment

Workers-rs is built using Rust and WebAssembly. These provide FFI compatibility for the Workers JS Runtime. 

- Install [Rust](https://www.rust-lang.org/tools/install).
- Install a code editor (e.g.[VS Code](https://code.visualstudio.com/)).
- Install the [git](https://git-scm.com/) version control tool.

### Install dependencies
- Install [Rust Wasm](https://rustwasm.github.io/)
- Install [miniflare](https://miniflare.dev/).

### Fork and clone this repository

Any contributions you make will be via [Pull Requests](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests) on [GitHub](https://github.com/) developed in a local git repository and pushed to your own fork of the repository.

- Ensure you have [created an account](https://docs.github.com/en/get-started/onboarding/getting-started-with-your-github-account) on GitHub.
- [Create your own fork](https://docs.github.com/en/get-started/quickstart/fork-a-repo) of [this repository](https://github.com/cloudflare/workers-rs).
- Clone your fork to your local machine
  ```sh
  > git clone https://github.com/<your-github-username>/workers-rs
  > cd workers-rs
  ```
  You can see that your fork is setup as the `origin` remote repository.
  Any changes you wish to make should be in a local branch that is then pushed to this origin remote.
  ```sh
  > git remote -v
  origin	https://github.com/<your-github-username>/workers-rs (fetch)
  origin	https://github.com/<your-github-username>/workers-rs (push)
  ```
- Add `cloudflare/workers-rs` as the `upstream` remote repository.
  ```sh
  > git remote add upstream https://github.com/cloudflare/workers-rs
  > git remote -v
  origin	https://github.com/<your-github-username>/workers-rs (fetch)
  origin	https://github.com/<your-github-username>/workers-rs (push)
  upstream	https://github.com/cloudflare/workers-rs (fetch)
  upstream	https://github.com/cloudflare/workers-rs (push)
  ```
- You should regularly pull from the `main` branch of the `upstream` repository to keep up to date with the latest changes to the project.
  ```sh
  > git switch main
  > git pull upstream main
  From https://github.com/cloudflare/workers-rs
  * branch            main       -> FETCH_HEAD
  Already up to date.
  ```

## Develop locally

### Project components

- **worker**: the user-facing crate, with Rust-familiar abstractions over the Rust<->JS/WebAssembly
  interop via wrappers and convenience library over the FFI bindings.
- **worker-sys**: Rust extern "C" definitions for FFI compatibility with the Workers JS Runtime.
- **worker-macros**: exports `event` and `durable_object` macros for wrapping Rust entry point in a
  `fetch` method of an ES Module, and code generation to create and interact with Durable Objects.
- **worker-sandbox**: a functioning Cloudflare Worker for testing features and ergonomics.
- **worker-build**: a cross-platform build command for `workers-rs`-based projects.
### Formatting

The code is checked for formatting errors by [rustfmt](https://github.com/rust-lang/rustfmt).

- Run the formatting checks
  ```sh
  > cargo fmt --all -- --check
  ```
- Fix formatting issues manually, or by running 
  ```sh
  > cargo fmt
  ```

### Linting

The code is checked for linting errors by [Clippy](https://github.com/rust-lang/rust-clippy).

- Run the linting checks
  ```sh
  > cargo clippy --all-features --all-targets --all -- -D warnings
  ```
All linter errors must be fixed for PRs to be merged.

### Testing

Make sure to have [wasmpack](https://rustwasm.github.io/) and [miniflare](https://miniflare.dev/) installed.

Once you're done making changes, you can start testing locally using [miniflare](https://miniflare.dev/).

1. Build bindgen test subject
```
cargo build --manifest-path worker-build/bindgen-test-subject/Cargo.toml --target 
```
2. Run builder tests
```
cargo test --manifest-path worker-build/Cargo.toml 
```
3. Build local worker-build 
  ```
  cargo install --path ./worker-build --force --debug
  ```
4. Run miniflare in the `worker-sandbox` directory
  ```
miniflare -c ./wrangler.toml --no-cf-fetch --no-update-check
  ```
5. In a new terminal, run all tests in the `worker-sandbox` directory
  ```
  cargo test
  ```
or run individual tests 
  ```
  cargo test test_name
  ```
## Steps for making changes using Git

Every change you make should be stored in a [git commit](https://github.com/git-guides/git-commit).
Changes should be committed to a new local branch, which then gets pushed to your fork of the repository on GitHub.

- Ensure your `main` branch is up to date
  ```sh
  > git switch main
  > git pull upstream main
  ```
- Create a new branch, based off the `main` branch
  ```sh
  > git checkout -b <new-branch-name> main
  ```
- Stage files to include in a commit
  - Use [VS Code](https://code.visualstudio.com/docs/editor/versioncontrol#_git-support)
  - Or add and commit files via the command line
  ```sh
  > git add <paths-to-changes-files>
  > git commit
  ```
- Push changes to your fork
  ```sh
  git push -u origin <new-branch-name>
  ```
- Once you are happy with your changes, create a Pull Request on GitHub

## Changesets

Every non-trivial change to the project - those that should appear in the changelog - must be captured in a "changeset".
We use the [changesets](https://github.com/changesets/changesets/blob/main/README.md) tool for creating changesets, publishing versions and updating the changelog.

- Create a changeset for the current change.
  ```sh
  > npx changeset
  ```
- Select which workspaces are affected by the change and whether the version requires a major, minor or patch release.
- Update the generated changeset with a description of the change.
- Include the generate changeset in the current commit.
  ```sh
  > git add ./changeset/*.md
  ```

### Changeset message format

Each changeset is a file that describes the change being merged. This file is used to generate the changelog when the changes are released.

To help maintain consistency in the changelog, changesets should have the following format:

```
<TYPE>: <TITLE>

<BODY>

[BREAKING CHANGES <BREAKING_CHANGE_NOTES>]
```

- `TYPE` should be a single word describing the "type" of the change. For example, one of `feature`, `fix`, `refactor`, `docs` or `chore`.
- `TITLE` should be a single sentence containing an imperative description of the change.
- `BODY` should be one or more paragraphs that go into more detail about the reason for the change and anything notable about the approach taken.
- `BREAKING_CHANGE_NOTES` (optional) should be one or more paragraphs describing how this change breaks current usage and how to migrate to the new usage.

### Changeset file example
The generated changeset file will contain the package name and type of change (eg. `patch`, `minor`, or `major`), followed by our changeset format described above.

Here's an example of a `patch` to the `worker` package, which provides a `docs` update:
```
---
"worker": patch
---

docs: updated the documentation for `Cache::get()` to reflect the behavior in the Cache runtime API

Our documentation should be consistent with the Workers runtime API documentation. Added information about the HTTP headers that will be respected in a get request to the Cache API.
```
# Contributing

Thanks for your interest in contributing to iconhunt.

## Development

1. Fork / clone the repository
2. Install a stable Rust toolchain (https://rustup.rs)
3. Create a branch: `git switch -c feat/your-change`
4. Make your change and verify locally:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --all
   cargo build --release
   ```
5. Commit and open a pull request

## Notes

- The tests never touch the network. `tests/common/mod.rs` serves recorded
  Iconify API responses from a local socket, and the CLI is pointed at it with
  `--api`.
- The Japanese keyword table lives in `src/ja.rs`. Add entries that map to the
  words icon sets actually use (`home`, `cog`, `trash`), and keep each entry
  to a few English keywords.
- License classes in `src/license.rs` are a reading aid, not legal advice.
  When adding an SPDX identifier, link the license text in the pull request.

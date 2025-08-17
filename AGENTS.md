# Agent Instructions

1. Run `./scripts/init.sh` to install the required tooling (`rustfmt`, `clippy`, and `cargo-machete`) via prebuilt binaries before starting work.
2. After making changes, run:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test`
   - `cargo machete`
3. Interpret user requests as actionable tasks and prefer complete pull request solutions over small snippets.
4. Remove or feature-gate unused code.
5. Account for potential voice-input typos.
6. All code comments and Markdown documentation must be in English; responses may be in Russian or English.
7. Choose the avatar persona that best fits the task and stay in character throughout the interaction. Avatar definitions are available at https://qqrm.github.io/avatars-mcp.

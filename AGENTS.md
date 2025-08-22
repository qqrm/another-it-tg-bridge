# Agent Instructions

1. Always roleplay as the "DevOps Engineer" avatar from the MCP server and stay in character throughout the interaction.
2. Before starting work, ensure the required tooling (`rustfmt`, `clippy`, `cargo-machete`, and `wrkflw`) is installed via `rustup` and `cargo` as needed.
3. After making changes, run:
   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test`
   - `cargo machete`
4. Interpret user requests as actionable tasks and prefer complete pull request solutions over small snippets.
5. Remove or feature-gate unused code.
6. Account for potential voice-input typos.
7. All code comments and Markdown documentation must be in English; responses may be in Russian or English.

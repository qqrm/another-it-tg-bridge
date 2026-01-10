# another-it-tg-bridge

Posts new articles from [another-it.ru](https://another-it.ru/) to the [another_it](https://t.me/another_it) Telegram channel.

## Quick Start

1. Clone the repository and enter it:
   ```sh
   git clone https://github.com/qqrm/another-it-tg-bridge.git
   cd another-it-tg-bridge
   ```
2. In **Settings → Secrets and variables → Actions**, add:
   - `TELEGRAM_BOT_TOKEN` – Telegram bot token.
   - `TELEGRAM_CHAT_ID` – target chat identifier.
   - (optional) `DEV_TELEGRAM_CHAT_ID` – chat for workflow failure notifications.
3. Enable GitHub Actions for the repository if it is disabled.
4. Trigger the workflow: open the **Actions** tab, select **Post to Telegram**, and click **Run workflow** (`workflow_dispatch`).

## Environment variables

- `TELEGRAM_BOT_TOKEN` – Telegram bot token.
- `TELEGRAM_CHAT_ID` – target chat identifier.
- `SENT_ARTICLES_PATH` – path to the JSON file that stores already sent URLs (`state.json` by default).
- `RUST_LOG` – optional logging level (e.g., `info`).
- `TARGET_DATE` – optional date in `YYYY/MM/DD` used to filter posts. The same value can be
  provided as the first command line argument for debugging.

## Manual run

```sh
export TELEGRAM_BOT_TOKEN=...
export TELEGRAM_CHAT_ID=...
# optional
export SENT_ARTICLES_PATH=state.json
cargo run --release
# For debugging a specific date:
# cargo run --release 2025/05/26
```

## GitHub Actions

The workflow at `.github/workflows/post.yml` builds the binary on a schedule or manual dispatch and commits the updated state file.
Rust toolchain updates are handled by Dependabot with auto-merge enabled for its pull requests.

## Development

Ensure the required development tools (`rustfmt`, `clippy`, `cargo-machete`, and `wrkflw`) are installed via `rustup` and `cargo` as needed.

## Origins

This project was bootstrapped from [`rust-hh-feed`](https://github.com/qqrm/rust-hh-feed) and retains a similar layout and Actions configuration for maintainers familiar with that repository.

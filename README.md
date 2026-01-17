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
   - `GOOGLE_CSE_API_KEY` – Google Custom Search API key.
   - `GOOGLE_CSE_CX` – Programmable Search Engine ID.
   - (optional) `DEV_TELEGRAM_CHAT_ID` – chat for workflow failure notifications.
3. Enable GitHub Actions for the repository if it is disabled.
4. Trigger the workflow: open the **Actions** tab, select **Post to Telegram**, and click **Run workflow** (`workflow_dispatch`).

## Environment variables

- `TELEGRAM_BOT_TOKEN` – Telegram bot token.
- `TELEGRAM_CHAT_ID` – target chat identifier.
- `GOOGLE_CSE_API_KEY` – Google Custom Search API key.
- `GOOGLE_CSE_CX` – Programmable Search Engine ID.
- `RUST_LOG` – optional logging level (e.g., `info`).

## Manual run

```sh
export TELEGRAM_BOT_TOKEN=...
export TELEGRAM_CHAT_ID=...
export GOOGLE_CSE_API_KEY=...
export GOOGLE_CSE_CX=...
cargo run --release
```

## GitHub Actions

The workflow at `.github/workflows/post.yml` builds the binary on a schedule or manual dispatch and caches the sent URL state.
Rust toolchain updates are handled by Dependabot with auto-merge enabled for its pull requests.

## Development

Ensure the required development tools (`rustfmt`, `clippy`, `cargo-machete`, and `wrkflw`) are installed via `rustup` and `cargo` as needed.

## Origins

This project was bootstrapped from [`rust-hh-feed`](https://github.com/qqrm/rust-hh-feed) and retains a similar layout and Actions configuration for maintainers familiar with that repository.

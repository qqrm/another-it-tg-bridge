# another-it-tg-bridge

A simple bridge that posts Another IT updates to Telegram.

## GitHub Actions

The workflow at `.github/workflows/notify.yml` builds and runs the binary on a schedule and commits any changes to `state.json`.

Configure the following repository secrets so the workflow can send messages:

- `TELEGRAM_BOT_TOKEN` – Telegram bot token.
- `TELEGRAM_CHAT_ID` – target chat identifier.

## Development

Run `./scripts/init.sh` to install the required development tools (`rustfmt`, `clippy`, and `cargo-machete`).

# another-it-tg-bridge — Specification

## 1) Purpose
The service publishes all posts from the daily page of Another IT to a Telegram channel or chat.

* **Debug mode (manual):** processes the date supplied as the workflow parameter.
* **Auto mode (scheduled):** processes strictly the previous day relative to UTC+3 (Moscow) at runtime.

## 2) Configuration (only 2 values)
From the `prod` environment in GitHub Actions:

* `TELEGRAM_BOT_TOKEN` — secret.
* `TELEGRAM_CHAT_ID` — variable.

No other configs, flags or variables exist.

## 3) Data source
Always read the daily page:

```
https://another-it.ru/YYYY/MM/DD/
```

The front page is never used.

## 4) Link extraction
* CSS selector: `main#site-content h2.entry-title > a`.
* Links are collected from top to bottom.
* Links are discarded if they do not contain the target date segment `YYYY/MM/DD`.

## 5) Sending order and format
* Send from oldest to newest (reverse before sending).
* Message format:

```
<Article title>
<Full article URL>
```

## 6) Date logic
* **Debug:** use the date from workflow input (`YYYY/MM/DD` or `YYYY-MM-DD`, normalized internally to `YYYY/MM/DD`).
* **Auto:** date is yesterday in UTC+3; format `YYYY/MM/DD`.

## 7) Behavior on empty result and errors
* If the daily page has no posts → log `no new posts`, exit 0.
* Network or HTTP errors for the daily page or articles → log warning and skip the problematic article; exit code is 0 if at least one message was sent, otherwise 1 (fatal daily page load error).
* Telegram API error (non-2xx) → log warning with HTTP status and continue; final exit code 0 if ≥1 message sent, else 1.

## 8) Application interface
* Optional positional argument — date (`YYYY/MM/DD` or `YYYY-MM-DD`).
  * If the argument is present → Debug mode.
  * If no argument → Auto mode.
* Reads `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` from environment.
* Exit codes: see section 7.

## 9) GitHub Actions — environment
* Both pipelines run on `ubuntu-latest`.
* `environment: prod` is set to pull in the secret and variable.
* The binary is taken from the latest release (asset named `another-it-tg-bridge`) and executed.

## 10) GitHub Actions — workflows
### 10.1 Auto (daily)
* **File:** `.github/workflows/post.yml`
* **Triggers:** `schedule` (once per day) and `workflow_dispatch` without inputs.
* **Steps:**
  1. `checkout`
  2. download asset `another-it-tg-bridge` from the latest release; `chmod +x`
  3. run the binary without arguments
  4. environment:
     * `TELEGRAM_BOT_TOKEN` from secrets
     * `TELEGRAM_CHAT_ID` from variables

### 10.2 Debug (manual)
* **File:** `.github/workflows/debug.yml`
* **Trigger:** `workflow_dispatch` with required `inputs.date`.
* **Steps:**
  1. `checkout`
  2. download asset `another-it-tg-bridge`; `chmod +x`
  3. run the binary with the single argument `inputs.date`
  4. environment:
     * `TELEGRAM_BOT_TOKEN` from secrets
     * `TELEGRAM_CHAT_ID` from variables

## 11) Acceptance criteria
* Debug mode requests `https://another-it.ru/<date>/` where `<date>` is the workflow parameter; all posts for that date are sent.
* Auto mode requests strictly the page for yesterday's date in UTC+3.
* Telegram messages are ordered from old to new and contain exactly two lines: title then URL.
* Only `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` from `prod` are used.
* If no posts exist, log `no new posts` and exit 0; the homepage is never requested.

## 12) Test cases (minimum)
1. **Debug / date with posts:** send N messages; exit 0.
2. **Debug / date without posts:** log `no new posts`; exit 0.
3. **Auto:** date corresponds to yesterday in UTC+3; send all posts for that day; exit 0.
4. **Daily page failure:** page unreachable → exit 1 (fatal).

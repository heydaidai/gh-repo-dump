# gh-repo-dump

Extract comprehensive GitHub repository details as structured JSON — all in one command.

## Features

- **Parallel API requests** — fetches 9 endpoints concurrently (repo, releases, tags, languages, contributors, branches, commits, readme, license)
- **Paginated data** — auto-paginates up to 3 pages (30 items/page) for list endpoints
- **Flexible output** — writes JSON to stdout or a file
- **Token support** — via `-t` flag or `GITHUB_TOKEN` environment variable
- **Timeout control** — per-endpoint timeout, configurable
- **Single binary** — zero runtime dependencies

## Installation

### From source (Rust)

```bash
cargo install --git https://github.com/ayang/gh-repo-dump
```

### Build locally

```bash
git clone https://github.com/ayang/gh-repo-dump
cd gh-repo-dump
cargo build --release
# Binary at ./target/release/gh-repo-dump
```

### Nix dev shell

```bash
nix develop
cargo build --release
```

## Usage

```bash
# Basic usage — dump rust-lang/rust to stdout
gh-repo-dump rust-lang/rust

# Save to file
gh-repo-dump rust-lang/rust -o rust.json

# With a GitHub token (recommended for higher rate limits)
export GITHUB_TOKEN=ghp_xxxx
gh-repo-dump rust-lang/rust

# Or pass token directly
gh-repo-dump rust-lang/rust -t ghp_xxxx

# Verbose mode, longer timeout
gh-repo-dump rust-lang/rust -v --timeout 60 -o repo.json
```

## Output Format

The output is a single JSON object with these top-level keys:

| Key           | Description                                |
|---------------|--------------------------------------------|
| `repo`        | Repository metadata (name, stars, forks, etc.) |
| `releases`    | List of releases (paginated)               |
| `tags`        | List of tags (paginated)                   |
| `languages`   | Language breakdown (bytes per language)    |
| `contributors`| Top contributors (paginated)               |
| `branches`    | Branch list (paginated)                    |
| `commits`     | Recent commits (paginated)                 |
| `readme`      | README file info (name, size, content in base64) |
| `license`     | License info (SPDX, name, body)            |

If any endpoint fails, its key contains `{"error": "..."}` instead, so the output is always complete.

## Requirements

- Rust toolchain (MSRV: 1.75+)
- Internet access (calls `api.github.com`)

## Notes

- Without a token, GitHub API rate limit is **60 requests/hour**. With a token, it's **5,000 requests/hour**.
- Paginated endpoints fetch at most **3 pages × 30 items = 90 entries** each. This covers most real-world use cases.
- The overall timeout is `3 × --timeout` (one multiplier for concurrent batch + safety margin).

## License

MIT

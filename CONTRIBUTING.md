# Contributing to UbiHome

We welcome any contributions to the UbiHome suite of code and documentation!

Please follow the semantic commit message format for all commits.

## Development

Just use the devcontainer setup.

```bash
sudo apt install -y musl-tools libdbus-1-dev pkg-config libasound2-dev
```

Vendored:

```
sudo apt-get install -y musl-tools
```

## Current Pitfalls

Logs are in `C:\Windows\System32\config\systemprofile\AppData\Local` as the service is running as `SYSTEM` user.

## Process

- Validate config.yaml
  - Before Build (not yet implemented)
  - During Runtime

## PR Automation

Every pull request is automatically checked by a [GitHub Agentic Workflow (gh-aw)](https://github.github.com/gh-aw/introduction/overview/) that runs on GitHub Actions.

### Documentation check

The workflow (`.github/workflows/pr-documentation-check.md`) checks whether code changes that introduce user-facing behaviour are also reflected in the documentation site (`documentation/`).

- **Triggers on**: `opened`, `synchronize`, `reopened` pull request events
- **Posts a comment** only when user-facing code changes are present but `documentation/` was not updated
- **Silent** (no comment) when everything looks good

If the automated comment is incorrect, add a reply to the PR comment explaining why the documentation update is not needed.

### Setup (maintainers only)

The workflow requires a `COPILOT_GITHUB_TOKEN` repository secret — a fine-grained Personal Access Token with the **Copilot Requests** permission:

> Repository → Settings → Secrets and variables → Actions → New repository secret
> - **Name**: `COPILOT_GITHUB_TOKEN`
> - **Value**: fine-grained PAT with **Copilot Requests** permission

See the [gh-aw engine reference](https://github.github.com/gh-aw/reference/engines/#github-copilot-default) for details.

After any change to the frontmatter of `pr-documentation-check.md`, regenerate the lock file to keep the hash in sync:

```bash
gh extension install github/gh-aw
gh aw compile .github/workflows/pr-documentation-check.md
git add .github/workflows/pr-documentation-check.lock.yml
git commit -m "chore(ci): regenerate pr-documentation-check lock file"
```

# Optimization

- https://github.com/johnthagen/min-sized-rust

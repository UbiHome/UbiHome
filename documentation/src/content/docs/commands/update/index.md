---
title: 'Updating'
description: 'Update to the latest release, a pre-release, or an in-development pull request build.'
---

`ubihome update` replaces the currently running executable in place. It works the same way whether UbiHome was [installed as a service](/commands/#install) or is just a standalone binary you invoke directly - either way, run the command from (or pointed at) the location of the executable you want replaced.

## Updating to the latest release

```bash
ubihome update
```

This:

1. Fetches the release list from the [GitHub Releases page](https://github.com/UbiHome/UbiHome/releases).
2. Picks the newest release (normal releases only, unless `--include-pre-release` is set - see below).
3. Prints the changelog for every release between your current version and that one.
4. Asks you to confirm before overwriting the current executable.
5. Downloads the build matching your current platform and replaces the running executable with it.

If you're already on the latest version, it prints `Already up to date!` and exits without prompting.

### Pre-releases

By default, only normal releases (tagged `vX.Y.Z`) are considered. To also update to pre-release/unstable tags (e.g. `vX.Y.Z-next.N`):

```bash
ubihome update --include-pre-release
```

## Updating to a pull request preview build

Every pull request's CI run publishes a build of UbiHome for each supported platform as a workflow artifact. `update pr` downloads the artifact from that PR's most recent **successful** CI run and installs it, which is useful for trying out a fix or feature before it's merged and released.

```bash
ubihome update pr <pr_number> --token <github_token>
```

:::caution
This installs an **unreleased development build** from an open pull request, which may be unstable or, if you don't trust the PR's author, unsafe. Only use this for pull requests you trust.
:::

### The GitHub token

Downloading workflow artifacts always requires an authenticated GitHub API request, even for public repositories like this one - there's no way around providing a token. The token needs at least:

- The `repo` scope (classic personal access tokens), or
- `actions:read` and `pull-requests:read` (fine-grained personal access tokens).

You can pass it either as `--token`, or via the `GITHUB_TOKEN` environment variable (the `--token` flag reads from it by default).

#### Using an automatically minted token via the GitHub CLI

If you have the [GitHub CLI](https://cli.github.com/) (`gh`) installed and are logged in (`gh auth login`), you don't need to create or manage a token yourself - `gh auth token` prints a valid token for your logged-in account on demand:

```bash
ubihome update pr 123 --token $(gh auth token)
```

Or, equivalently, via the environment variable:

```bash
GITHUB_TOKEN=$(gh auth token) ubihome update pr 123
```

This token is scoped to your own GitHub session and is never persisted by UbiHome - it's only used for the duration of the update command.

<!-- Backlinks to be displayed  -->
<div style="display:none" aria-hidden="true">
  <a href="/commands/">CLI</a>
</div>

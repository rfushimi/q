# Setting up the Homebrew Tap

This document describes how to set up and maintain the Homebrew tap for the `q` CLI tool.

## Initial Setup

1. Create a new GitHub repository named `homebrew-tools` (the `homebrew-` prefix is required by Homebrew)
2. Create the following directory structure:
   ```
   homebrew-tools/
   └── Formula/
       └── q.rb
   ```
3. Copy the q.rb formula file into the Formula directory

## GitHub Actions Setup

The release workflow is configured to automatically update the Homebrew tap when a new version is released. To set this up:

1. Create a new Personal Access Token (PAT) with `repo` scope:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Generate a new token with `repo` scope
   - Copy the token value

2. Add the PAT as a repository secret:
   - Go to the main q repository settings
   - Navigate to Secrets and variables → Actions
   - Add a new secret named `TAP_GITHUB_TOKEN` with the PAT value

## How it Works

When you push a new tag to the main repository:

1. The release workflow creates a new GitHub release
2. It builds and uploads the binary
3. It creates a source tarball and calculates its SHA256
4. It automatically updates the formula in rfushimi/homebrew-tools with:
   - The new version number
   - The correct tarball URL
   - The calculated SHA256 hash

## Manual Installation

Users can install q using:

```bash
brew tap rfushimi/tools
brew install q
```

## Troubleshooting

If the automatic update fails:

1. Check the GitHub Actions logs in the main repository
2. Verify the `TAP_GITHUB_TOKEN` has the correct permissions
3. Ensure the homebrew-tools repository exists and is accessible
4. Check that the Formula/q.rb file exists in the tap repository

## Version Updates

The version is automatically managed by the GitHub Actions workflow. When you want to release a new version:

1. Update the version in Cargo.toml
2. Create and push a new tag:
   ```bash
   git tag v0.1.0  # Use appropriate version
   git push origin v0.1.0
   ```

The workflow will handle the rest automatically.

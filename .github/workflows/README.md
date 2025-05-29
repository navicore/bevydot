# GitHub Actions Workflows

## Release Workflows

- **Trigger**: When a release is created
- **Behavior**: Automatically updates Cargo.toml and Cargo.lock to match the release tag
- **Setup Required**:
  1. Create a Personal Access Token with `repo` scope
  2. Add secrets:
     - `PAT`: Your Personal Access Token (optional, falls back to GITHUB_TOKEN)
     - `CRATES_IO_TOKEN`: Your crates.io API token

## Usage

## Version Tag Format

The workflows expect tags in the format `vX.Y.Z` (e.g., `v0.7.0`, `v1.0.0`).
The `v` prefix is automatically stripped when updating Cargo.toml.

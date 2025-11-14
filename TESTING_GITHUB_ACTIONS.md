# Testing GitHub Actions Locally

This guide explains how to test the GitHub Actions workflows locally before pushing to GitHub.

## Tool: `act`

[`act`](https://github.com/nektos/act) is a tool that runs GitHub Actions locally using Docker. It reads your workflow files and simulates the GitHub Actions environment.

### Installation

**macOS (Homebrew):**
```bash
brew install act
```

**Linux:**
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**Windows (Chocolatey):**
```bash
choco install act-cli
```

**Or download from releases:**
https://github.com/nektos/act/releases

### Prerequisites

- Docker must be installed and running
- Sufficient disk space for Docker images (act uses large images)

## Testing the Release Workflow

### 1. List Available Jobs

First, see what jobs are defined in your workflow:

```bash
act -l -W .github/workflows/release.yml
```

This shows all jobs without running them.

### 2. Test the Docker Assets Job (Recommended First Test)

The docker-assets job is the simplest and fastest to test:

```bash
# Dry run (shows what would happen)
act release -W .github/workflows/release.yml -j docker-assets --dryrun

# Actually run it
act release -W .github/workflows/release.yml -j docker-assets
```

This creates the `nstimes-docker-assets.tar.gz` file locally.

### 3. Test a Single Platform Build

Testing all 5 platforms takes a long time. Test one platform first:

**Test macOS ARM64 (if you're on Apple Silicon):**
```bash
act release -W .github/workflows/release.yml -j build --matrix platform.target:aarch64-apple-darwin
```

**Test Linux x64 (fastest, most compatible):**
```bash
act release -W .github/workflows/release.yml -j build --matrix platform.target:x86_64-unknown-linux-musl
```

**Test Windows x64:**
```bash
act release -W .github/workflows/release.yml -j build --matrix platform.target:x86_64-pc-windows-msvc
```

### 4. Test All Builds (Warning: Takes a Long Time)

```bash
act release -W .github/workflows/release.yml
```

This will run all 5 platform builds + docker assets job. Expect this to take 30-60 minutes.

## Common Options

### Use a Smaller Docker Image

By default, act uses large Ubuntu images. You can use smaller images:

```bash
# Use medium image (faster, but some tools missing)
act -P ubuntu-latest=catthehacker/ubuntu:act-latest -j docker-assets

# Use small image (fastest, minimal tools)
act -P ubuntu-latest=node:16-buster-slim -j docker-assets
```

### Set Environment Variables

If your workflow needs environment variables:

```bash
act release -W .github/workflows/release.yml --secret GITHUB_TOKEN=fake_token
```

### Verbose Output

See detailed logs:

```bash
act release -W .github/workflows/release.yml -j docker-assets -v
```

### Skip Docker Pull

If you've already pulled the images:

```bash
act release --pull=false
```

## Limitations of `act`

1. **Not 100% accurate**: Some things work differently than real GitHub Actions
2. **Slow**: Uses Docker containers which can be slow
3. **Disk space**: Requires several GB for Docker images
4. **Matrix builds**: Testing all matrix combinations takes a very long time
5. **Upload artifacts**: Won't actually upload to GitHub releases (but creates files locally)

## Recommended Testing Strategy

### For Quick Iteration

1. Test docker-assets job first (fastest)
2. Test one platform build to verify compilation
3. Fix any issues
4. Push to GitHub and let Actions run all platforms

```bash
# Quick test workflow
act release -W .github/workflows/release.yml -j docker-assets
act release -W .github/workflows/release.yml -j build --matrix platform.target:x86_64-unknown-linux-musl
```

### For Thorough Testing

Only run full matrix builds when you're confident:

```bash
# Full test (takes 30-60 minutes)
act release -W .github/workflows/release.yml
```

## Alternative: GitHub Actions Debugging

Instead of testing locally, you can test on GitHub with draft releases:

1. Create a draft release on GitHub
2. Watch the Actions run
3. Download artifacts to verify
4. Delete the draft release if there are issues
5. Fix and repeat

This is often faster than running `act` locally for complex matrix builds.

## Troubleshooting

### "Docker daemon not running"
- Start Docker Desktop
- Run: `docker ps` to verify Docker is working

### "Permission denied"
- Run with sudo: `sudo act ...`
- Or add your user to docker group: `sudo usermod -aG docker $USER`

### "Image not found"
- Let act download the default image: Just run `act` once
- Or specify an image: `act -P ubuntu-latest=ubuntu:latest`

### Build fails but works in GitHub Actions
- Check Docker image has required tools
- Use the full-size image: `act -P ubuntu-latest=catthehacker/ubuntu:full-latest`
- Some features only work on real GitHub Actions

## Files Created Locally

When you run `act`, it creates files in your local directory:

- `nstimes-*.tar.gz` or `nstimes-*.zip` - Build artifacts
- `nstimes-docker-assets.tar.gz` - Docker deployment files

These are **not** uploaded to GitHub when running locally (the upload step is simulated).

## Summary

**Quick test:**
```bash
act release -W .github/workflows/release.yml -j docker-assets --dryrun
act release -W .github/workflows/release.yml -j docker-assets
```

**Medium test (one platform):**
```bash
act release -W .github/workflows/release.yml -j build --matrix platform.target:x86_64-unknown-linux-musl
```

**Full test (all platforms, slow):**
```bash
act release -W .github/workflows/release.yml
```

For most iteration, test locally once to verify basic functionality, then use GitHub's Actions for full matrix builds.

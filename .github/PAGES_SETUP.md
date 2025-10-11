# GitHub Pages Setup Guide

This guide explains how to set up GitHub Pages for the SpatioLite documentation.

## Quick Setup

1. **Navigate to your repository settings**
   - Go to your SpatioLite repository on GitHub
   - Click the "Settings" tab

2. **Enable GitHub Pages**
   - In the left sidebar, click "Pages"
   - Under "Source", select "GitHub Actions"
   - Click "Save"

3. **Verify the setup**
   - Push a commit to the `main` branch
   - Check the "Actions" tab to see the documentation workflow running
   - Once complete, your documentation will be available at: `https://<username>.github.io/SpatioLite/`

## Troubleshooting

### Error: "Get Pages site failed"

This error occurs when GitHub Pages is not properly configured. Follow these steps:

1. **Check Pages Configuration**
   - Go to Settings → Pages
   - Ensure "Source" is set to "GitHub Actions" (not "Deploy from a branch")

2. **Verify Repository Permissions**
   - The repository must be public, OR
   - You must have GitHub Pro/Team/Enterprise for private repository Pages

3. **Check Workflow Permissions**
   - Go to Settings → Actions → General
   - Under "Workflow permissions", ensure "Read and write permissions" is selected
   - OR ensure the workflow has explicit `pages: write` and `id-token: write` permissions

## Deployment Strategy

Documentation is now deployed only on:
- **Releases**: When you publish a new release
- **Manual trigger**: Using workflow_dispatch in GitHub Actions

This prevents unnecessary deployments on every commit to main and ensures
documentation updates align with actual releases.

### Error: "Resource not accessible by integration"

This typically means:

1. **Missing Permissions**: The workflow needs `pages: write` and `id-token: write` permissions
2. **Pages Not Enabled**: GitHub Pages must be enabled in repository settings
3. **Wrong Source**: Pages source must be set to "GitHub Actions"

## Manual Verification

You can verify the setup works by:

1. **Running the documentation workflow manually:**
   - Go to Actions → Documentation
   - Click "Run workflow"
   - Select the `main` branch
   - Click "Run workflow"

2. **Publishing a release:**
   - Go to Releases → Create a new release
   - This will automatically trigger documentation deployment

3. **Check the workflow logs for any errors**

4. **Once successful, visit your Pages URL to see the documentation**

The documentation includes:
- **API Documentation**: Comprehensive Rust docs with examples
- **User Guides**: Getting started, spatial queries, trajectory tracking
- **Examples**: Runnable code samples demonstrating features

## What Gets Deployed

The documentation workflow builds and deploys:

- **API Documentation**: Comprehensive Rust docs with detailed examples and method documentation
- **User Guide**: Complete guide covering basic to advanced operations
- **Code Examples**: Multiple focused examples (getting_started, spatial_queries, trajectory_tracking)
- **Geometry Operations**: Full geometry support documentation with WKT examples
- **Performance Information**: Benchmark results and optimization guides

## Updating Documentation

Documentation is automatically updated when:

- **A new release is published** (recommended)
- **The workflow is manually triggered** via workflow_dispatch

Note: Documentation is no longer deployed on every push to main. This ensures
that published documentation aligns with stable releases and reduces unnecessary
deployment overhead.

The documentation includes:
- Auto-generated API docs from Rust code
- Hand-written guides and tutorials
- Live code examples that are tested in CI
- Performance metrics and benchmarks

## Local Development

To work on documentation locally:

```bash
# Install mdbook
cargo install mdbook

# Build and serve documentation
cd docs
mdbook serve --open
```

This will start a local server at `http://localhost:3000` where you can preview changes.
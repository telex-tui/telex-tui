# Book Deployment

**Status:** Setup ready, deployment disabled until public launch.

The Telex Book will be automatically deployed to GitHub Pages once the repository is made public.

## Pre-Launch: Local Development Only

For now, preview the book locally:

```bash
cd docs/book
mdbook serve --open
```

Visit http://localhost:3000 to see your changes.

## Post-Launch Setup

When ready to make the repository public and deploy the book:

### 1. Make Repository Public

1. Go to repository Settings → General
2. Scroll to "Danger Zone"
3. Click "Change visibility" → "Make public"

### 2. Enable GitHub Pages

1. Go to repository Settings → Pages
2. Set Source to "GitHub Actions"
3. Save

### 3. Workflow

The `.github/workflows/book.yml` workflow:
- Triggers on push to `main` (when book files change)
- Installs mdBook v0.4.40
- Builds the book from `docs/book/`
- Deploys to GitHub Pages

### 4. Access the Book

Once deployed, the book will be available at:
```
https://telex-tui.github.io/telex-tui/
```

Or if using a custom domain, configure in Settings → Pages.

**Note:** GitHub Pages only works for public repositories (or with GitHub Enterprise). The workflow is ready but won't activate until the repo is public.

## Manual Deployment

To manually trigger a deployment:

1. Go to Actions tab
2. Select "Deploy Book" workflow
3. Click "Run workflow"
4. Select the `main` branch
5. Click "Run workflow"

## Local Preview

To preview the book locally before pushing:

```bash
cd docs/book
mdbook serve
```

Then visit http://localhost:3000

## Build Directory

The built book is output to `docs/book/book/` (configured in `book.toml`).

This directory is git-ignored and only used for deployment.

## Troubleshooting

**Workflow fails with "mdbook: command not found"**
- The workflow installs mdBook automatically
- If running locally, install with: `cargo install mdbook`

**Changes don't appear on the site**
- Check Actions tab to see if deployment succeeded
- GitHub Pages can take 1-2 minutes to update after deployment
- Hard refresh your browser (Ctrl+Shift+R / Cmd+Shift+R)

**404 on the book URL**
- Verify GitHub Pages is enabled in Settings → Pages
- Check that Source is set to "GitHub Actions"
- Ensure the workflow completed successfully

## URL Structure

The book will be available at the root of the GitHub Pages site:
- Main page: `https://telex-tui.github.io/telex-tui/`
- Getting Started: `https://telex-tui.github.io/telex-tui/getting-started/installation.html`
- Etc.

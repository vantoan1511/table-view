---
description: Bump version, commit, tag, and push to trigger automated release workflow
---

# Automated Release Workflow

This workflow ensures that when the user asks to "release a new version", all necessary files are updated, a git tag is created, and pushed to the repository to trigger the automated CI/CD release workflow.

## Step 1: Bump Version
Identify the target version:
- If the user specifies a version (e.g., "release v1.0.0"), use that exact version.
- Otherwise, identify the next logical semantic version bump (e.g., from `0.0.17` to `0.0.18`).
Update the version strings in the following files:
- `package.json`
- `neutralino.config.json`

## Step 2: Update Changelog
Prepare the `CHANGELOG.md` for the new release:
- Move all entries under `## [Unreleased]` to a new version section: `## [<NEW_VERSION>] - <YYYY-MM-DD>`.
- Create a new empty `## [Unreleased]` section at the top.

## Step 3: Commit Changes
Stage and commit the version bumps and changelog updates.

// turbo
```powershell
git add package.json neutralino.config.json CHANGELOG.md
git commit -m "chore: release v<NEW_VERSION>"
```
*(Replace `<NEW_VERSION>` with the actual new version, e.g. `0.0.18`)*

## Step 4: Push Commit
Push the new release commit to the remote branch.

// turbo
```powershell
git push
```

## Step 5: Create and Push Tag
Create a git tag for the new version and push it to the remote repository. Pushing the tag will automatically trigger the GitHub Actions release workflow.

// turbo
```powershell
git tag v<NEW_VERSION>
git push --tags
```
*(Replace `<NEW_VERSION>` with the actual new version, e.g. `0.0.18`)*

## Summary
Once `git push --tags` completes, inform the user that the tag was pushed and the automated release workflow has been triggered.

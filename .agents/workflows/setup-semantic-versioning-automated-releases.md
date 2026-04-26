---
description: Automate version bumps and changelog generation
---

1. **Install semantic-release**:
   - Automate versioning based on commit messages.
   // turbo
   - Run `npm install --save-dev semantic-release @semantic-release/changelog @semantic-release/git`

2. **Configure Commit Convention**:
   - Use Conventional Commits.
   ```bash
   # Format: <type>(<scope>): <description>
   feat: add dark mode support
   fix: resolve hydration error
   docs: update README
   chore: upgrade dependencies
   ```

3. **Create Release Config**:
   - Create `.releaserc.json`.
   ```json
   {
     "branches": ["main"],
     "plugins": [
       "@semantic-release/commit-analyzer",
       "@semantic-release/release-notes-generator",
       "@semantic-release/changelog",
       "@semantic-release/npm",
       "@semantic-release/github",
       ["@semantic-release/git", {
         "assets": ["CHANGELOG.md", "package.json"],
         "message": "chore(release): ${nextRelease.version} [skip ci]"
       }]
     ]
   }
   ```

4. **Setup GitHub Actions**:
   - Automate releases on merge to main.
   ```yaml
   # .github/workflows/release.yml
   name: Release
   on:
     push:
       branches: [main]
   jobs:
     release:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - uses: actions/setup-node@v3
         - run: npm ci
         - run: npx semantic-release
           env:
             GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
   ```

5. **Commit with Convention**:
   - Follow the format.
   ```bash
   git commit -m "feat: add user authentication"
   # Will bump MINOR version (0.1.0 -> 0.2.0)
   
   git commit -m "fix: resolve login bug"
   # Will bump PATCH version (0.2.0 -> 0.2.1)
   
   git commit -m "feat!: redesign API\n\nBREAKING CHANGE: API endpoints changed"
   # Will bump MAJOR version (0.2.1 -> 1.0.0)
   ```

6. **Pro Tips**:
   - Use Commitizen for interactive commit messages: `npx cz`.
   - CHANGELOG.md is auto-generated, don't edit manually.
   - Releases create Git tags automatically.
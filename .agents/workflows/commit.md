---
description: Automatic commit message generator and fast AI-powered commit for all current changes
---

// turbo-all

This workflow automatically stages all changes, generates a descriptive commit message, and commits them in one go.

### Steps:

1. **Synchronize Documentation**: Run the `@/sync-docs` workflow to ensure `PROGRESS.md` and `SPEC.md` are up to date with the latest changes.
2. **Ensure Production Settings**: Update configuration files for production mode before committing.
   - In `neutralino.config.json`, set `"enableInspector": false`, set `"tokenSecurity": "one-time"`.
   - In `index.html`, ensure script source path is `"%PUBLIC_URL%/__neutralino_globals.js"`.
3. **Stage All Changes**: Automatically stage all modified and new files.
   ```bash
   git add .
   ```
4. **Analyze Changes**: Get the diff of staged changes to understand the context.
   ```bash
   git diff --cached
   ```
5. **Generate & Commit**: Generate a professional message following [Conventional Commits](https://www.conventionalcommits.org/) and execute the commit.
   ```bash
   git commit -m "<ai_generated_message>"
   ```
6. **Push**: Optionally push the changes.
   ```bash
   git push
   ```
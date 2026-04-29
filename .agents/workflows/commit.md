---
description: Automatic commit message generator and fast AI-powered commit for all current changes
---

// turbo-all

This workflow automatically stages all changes, generates a descriptive commit message, and commits them in one go.

### Steps:

1. **Ensure Production Settings**: Update configuration files for production mode before committing.
   - In `neutralino.config.json`, set `"enableInspector": false`, set `"tokenSecurity": "one-time"`.
   - In `index.html`, ensure script source path is `"%PUBLIC_URL%/__neutralino_globals.js"`.
2. **Stage All Changes**: Automatically stage all modified and new files.
   ```bash
   git add .
   ```
2. **Analyze Changes**: Get the diff of staged changes to understand the context.
   ```bash
   git diff --cached
   ```
3. **Generate & Commit**: Generate a professional message following [Conventional Commits](https://www.conventionalcommits.org/) and execute the commit.
   ```bash
   git commit -m "<ai_generated_message>"
   ```
4. **Push**: Optionally push the changes.
   ```bash
   git push
   ```
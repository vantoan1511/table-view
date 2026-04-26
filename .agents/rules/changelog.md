# Release & Changelog Standards

## Core Principles
- **Clarity Over Verbosity**: Changelogs must be written for users and stakeholders, not just developers. Use clear, non-technical language where possible.
- **Contextual Meaning**: Every entry should explain *why* a change was made or *how* it benefits the user, rather than just stating *what* was changed.

## Workflow Requirements
- **Conventional Commits**: All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This ensures automated tools can correctly categorize changes.
- **Enriched GitHub Releases**: When creating a GitHub Release, the description must include:
    - **🚀 Highlights**: A 1-2 sentence summary of the most important changes in this version.
    - **✨ New Features**: Grouped `feat` commits with descriptive titles.
    - **🐛 Bug Fixes**: Grouped `fix` commits with explanations of what was corrected.
    - **⚡ Performance & Polish**: Highlights of `perf`, `refactor`, and `style` changes that improve the UX.
    - **⚠️ Breaking Changes**: A clearly marked section for any breaking changes with migration instructions if necessary.
- **Progress Sync**: Every release must trigger an update to `PROGRESS.md` to move completed items to "Milestones Reached".

## Formatting
- Use emojis to make the changelog visually scannable.
- Use subheadings for different categories of changes.
- Provide links to issue numbers or PRs if applicable.

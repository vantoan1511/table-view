# Documentation Synchronization Workflow

This workflow ensures that the project's progress tracking and technical specifications remain in sync with the actual codebase.

### Steps:

1. **Analyze Recent Changes**: Review the git diff and recent file modifications to understand what has changed since the last documentation update.
2. **Synchronize PROGRESS.md**:
   - Update the "Current Status" if milestones were reached.
   - Mark completed tasks in the "Upcoming Tasks" section.
   - Add new phases or milestones for major architectural changes.
   - Ensure technology references (e.g., Rust vs Go) are accurate.
3. **Synchronize SPEC.md**:
   - Update architecture diagrams or descriptions.
   - Document new protocol fields or API endpoints.
   - Reflect changes in build tools, environment management, or deployment strategies.
4. **Synchronize README.md**:
   - Update the tech stack (e.g., Rust vs Go).
   - Refresh the feature list with latest additions (e.g., Real-time Telemetry).
   - Update version badges or installation instructions if necessary.
   - Ensure screenshots or mockups are current.
5. **Final Review**: Ensure all documentation is accurate, professional, and reflects the current state of the repository.

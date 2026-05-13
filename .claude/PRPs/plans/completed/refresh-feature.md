# Implementation Plan: Entity Refresh Feature

## Summary
Implement a unified "Refresh" feature across all database entities (Connection, Database, Schema, and Table) accessible via right-click context menus and the `F5` shortcut.

## User Story
As a database developer, I want to refresh the state of my database entities (tables, schemas, databases) so that I can see the latest changes made by other users or scripts without restarting the application.

## Problem → Solution
- **Current State**: Refresh is only available for the Connection level in the sidebar, and `Ctrl+R` refreshes table data. No context menu exists for Database, Schema, or Table nodes.
- **Desired State**: All nodes in the database tree support a right-click context menu with a "Refresh" action. `F5` triggers a contextual refresh based on the active tab or selected tree node.

## Metadata
- **Complexity**: Medium
- **Target Files**: 
    - `src/components/sidebar/DatabaseTree.vue` (Orchestrator)
    - `src/components/sidebar/DatabaseNode.vue` (Emitter)
    - `src/components/sidebar/SchemaNode.vue` (Emitter)
    - `src/components/sidebar/ObjectGroupNode.vue` (Emitter)
    - `src/components/sidebar/DatabaseContextMenu.vue` (NEW)
    - `src/components/sidebar/SchemaContextMenu.vue` (NEW)
    - `src/components/sidebar/TableContextMenu.vue` (NEW)
    - `src/composables/useKeyboardShortcuts.ts` (Shortcut handler)
    - `src/stores/schema.ts` (State actions)

---

## UX Design

### Before
- Connection: Right-click -> Menu with "Refresh Schema".
- Database/Schema/Table: Right-click -> Browser default menu (blocked).
- `F5`: Blocked or browser reload.

### After
- Connection: Right-click -> Existing menu with "Refresh".
- Database: Right-click -> Menu with "Refresh" (and placeholders for Create/Delete).
- Schema: Right-click -> Menu with "Refresh" (and placeholders for Create/Delete).
- Table: Right-click -> Menu with "Refresh" (and placeholders for Alter/Delete).
- `F5`: Refreshes active table tab OR refreshes the active connection tree.

---

## Mandatory Reading

| Priority | File | Why |
|---|---|---|
| P0 | `src/components/sidebar/DatabaseTree.vue` | Main tree management and context menu handling. |
| P1 | `src/stores/schema/useSchemaActions.ts` | Existing refresh logic (`loadSchema`, `refreshDbSchema`). |
| P1 | `src/composables/useKeyboardShortcuts.ts` | Shortcut registration logic. |

---

## Proposed Changes

### [NEW] [DatabaseContextMenu.vue](file:///d:/Projects/table-view/src/components/sidebar/DatabaseContextMenu.vue)
A component to show actions for a Database node.
- Items: "Refresh", "Create Schema" (disabled), "Delete" (disabled).

### [NEW] [SchemaContextMenu.vue](file:///d:/Projects/table-view/src/components/sidebar/SchemaContextMenu.vue)
A component to show actions for a Schema node.
- Items: "Refresh", "Create Table" (disabled), "Delete" (disabled).

### [NEW] [TableContextMenu.vue](file:///d:/Projects/table-view/src/components/sidebar/TableContextMenu.vue)
A component to show actions for a Table node.
- Items: "Refresh", "Alter Table" (disabled), "Delete Table" (disabled).

### [MODIFY] [DatabaseTree.vue](file:///d:/Projects/table-view/src/components/sidebar/DatabaseTree.vue)
- Add state for multiple context menu types (or a generic one).
- Implement `onEntityContextMenu(event, type, payload)` to capture which entity was clicked.
- Handle "refresh" action for each type:
    - `connection`: Existing logic.
    - `database`: Call `schemaStore.refreshDbSchema`.
    - `schema`: Call `schemaStore.loadDbSchema` (force reload).
    - `table`: Call `gridStore.loadTable` if that table is active, and refresh metadata.

### [MODIFY] [useKeyboardShortcuts.ts](file:///d:/Projects/table-view/src/composables/useKeyboardShortcuts.ts)
- Add `F5` key listener.
- Logic for `F5`:
    ```typescript
    if (e.key === 'F5') {
      e.preventDefault();
      if (tabsStore.activeTab?.type === TabType.TABLE) {
        gridStore.loadTable(tabsStore.activeTab.tableName);
      } else {
        // Fallback to refreshing the tree for active connection
        schemaStore.loadSchema(undefined, connectionsStore.activeConnectionId);
      }
    }
    ```

### [MODIFY] [DatabaseNode.vue](file:///d:/Projects/table-view/src/components/sidebar/DatabaseNode.vue), [SchemaNode.vue](file:///d:/Projects/table-view/src/components/sidebar/SchemaNode.vue), [ObjectGroupNode.vue](file:///d:/Projects/table-view/src/components/sidebar/ObjectGroupNode.vue)
- Add `@contextmenu.prevent` to the main row elements.
- Emit `contextmenu` event to parent (`DatabaseTree.vue`).

---

## Step-by-Step Tasks

### Task 1: Create Context Menu Components
- **ACTION**: Create `DatabaseContextMenu.vue`, `SchemaContextMenu.vue`, and `TableContextMenu.vue` in `src/components/sidebar/`.
- **IMPLEMENT**: Use `ContextMenu.vue` as the base. Add a "Refresh" button with `RefreshCw` icon.
- **MIRROR**: `ConnectionContextMenu.vue` patterns.

### Task 2: Update Schema Store Actions
- **ACTION**: Ensure `refreshDbSchema` and `loadSchema` can be used to refresh specific nodes.
- **VALIDATE**: Verify that calling `loadDbSchema` with an existing entry in cache refreshes it from the bridge.

### Task 3: Implement Context Menu Logic in DatabaseTree
- **ACTION**: Add `contextMenu` state for each entity type.
- **IMPLEMENT**: Update `onContextMenu` to accept an `entityType` and `data` (connectionId, dbName, schemaName, tableName).
- **VALIDATE**: Right-clicking different nodes opens the correct menu.

### Task 4: Link Context Menus to Nodes
- **ACTION**: Update `DatabaseNode`, `SchemaNode`, and `ObjectGroupNode` to emit context menu events.
- **IMPLEMENT**: Pass props up to `DatabaseTree`.

### Task 5: Add F5 Shortcut
- **ACTION**: Update `useKeyboardShortcuts.ts`.
- **IMPLEMENT**: Add `F5` support. Map it to the most relevant refresh action.

---

## Testing Strategy
- **Manual Verification**:
    1. Right-click a Connection -> Click Refresh -> Verify tree updates.
    2. Right-click a Database -> Click Refresh -> Verify schemas/tables inside are reloaded.
    3. Right-click a Schema -> Click Refresh -> Verify tables inside are reloaded.
    4. Right-click a Table -> Click Refresh -> Verify table data (if open) and metadata are reloaded.
    5. Press F5 with a table tab open -> Verify table data refreshes.
    6. Press F5 with no tab open -> Verify sidebar tree refreshes.

## Acceptance Criteria
- [ ] Context menu appears for all 4 entity types on right-click.
- [ ] "Refresh" action is functional for all entity types.
- [ ] Placeholder items (Create/Delete/Alter) are present but disabled or marked as upcoming.
- [ ] F5 key refreshes the active context correctly.
- [ ] No browser default context menu appears on these entities.

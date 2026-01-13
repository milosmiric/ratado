# Ratado Implementation Plan

This directory contains the phased implementation plan for Ratado, broken into manageable story files.

## How to Use

1. Start a new Claude session
2. Reference the phase file you want to work on
3. Say: `Let's work on Story X.Y from /docs/plan/phase-XX-name.md`
4. Claude will read the relevant docs and implement the story
5. Mark stories as complete with `[x]` when done

## Phase Overview

| Phase | File | Stories | Focus |
|-------|------|---------|-------|
| 1 | [phase-01-foundation.md](phase-01-foundation.md) | 4 | Cargo setup, modules, models |
| 2 | [phase-02-storage.md](phase-02-storage.md) | 4 | Turso database, migrations, CRUD |
| 3 | [phase-03-core-ui.md](phase-03-core-ui.md) | 5 | App state, layout, widgets |
| 4 | [phase-04-input-handling.md](phase-04-input-handling.md) | 4 | Events, commands, keybindings |
| 5 | [phase-05-task-operations.md](phase-05-task-operations.md) | 5 | Add, edit, delete, toggle tasks |
| 6 | [phase-06-projects-tags.md](phase-06-projects-tags.md) | 3 | Project & tag management |
| 7 | [phase-07-filtering-search.md](phase-07-filtering-search.md) | 4 | Filters, sorting, search |
| 8 | [phase-08-additional-views.md](phase-08-additional-views.md) | 3 | Help, detail, calendar views |
| 9 | [phase-09-notifications.md](phase-09-notifications.md) | 3 | Reminders, notifications |
| 10 | [phase-10-cli.md](phase-10-cli.md) | 3 | Command-line interface |
| 11 | [phase-11-configuration.md](phase-11-configuration.md) | 2 | Config file, themes |
| 12 | [phase-12-testing-polish.md](phase-12-testing-polish.md) | 4 | Tests, error handling |

**Total: 44 stories across 12 phases**

## Recommended Order

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5
   ↓
Minimum Viable Product (can add/edit/delete/complete tasks)
   ↓
Phase 6 → Phase 7 → Phase 8 → Phase 9 → Phase 10 → Phase 11 → Phase 12
```

## Dependencies

- **Phase 2** requires Phase 1 (models needed for storage)
- **Phase 3** requires Phase 2 (storage needed to load data)
- **Phase 4** requires Phase 3 (UI needed for input handling)
- **Phase 5** requires Phase 4 (handlers needed for task operations)
- **Phases 6-12** can be done in any order after Phase 5

## Progress Tracking

Update this section as phases are completed:

- [x] Phase 1: Project Foundation
- [x] Phase 2: Storage Layer
- [x] Phase 3: Core UI
- [x] Phase 4: Input Handling
- [ ] Phase 5: Task Operations
- [ ] Phase 6: Projects & Tags
- [ ] Phase 7: Filtering & Search
- [ ] Phase 8: Additional Views
- [ ] Phase 9: Notifications
- [ ] Phase 10: CLI Interface
- [ ] Phase 11: Configuration
- [ ] Phase 12: Testing & Polish

## Quick Start Example

```
You: Let's work on Story 1.1 from /docs/plan/phase-01-foundation.md

Claude: [Reads the phase file and implements Story 1.1]
```

## Related Documentation

- [SPECIFICATION.md](../SPECIFICATION.md) - Full requirements
- [UI_MOCKUPS.md](../UI_MOCKUPS.md) - UI designs
- [TESTING_STRATEGY.md](../TESTING_STRATEGY.md) - Testing approach

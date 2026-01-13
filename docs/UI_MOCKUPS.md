# Ratado UI Mockups

This document shows the planned terminal UI screens and layouts.

---

## 1. Main View (Default)

The primary interface with sidebar and task list.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado v0.1.0                                       [!2 overdue] [Today: 4] │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │                                                         │
│  PROJECTS         │  TASKS                                    ↑↓ Navigate  │
│  ─────────        │  ─────────────────────────────────────────────────────  │
│  ● Inbox      (3) │                                                         │
│ ›● Work       (5) │  ▸ [ ] !! Review PR #142            Due: Today   @work  │
│    Personal   (2) │    [ ] !! Fix login bug             Due: Today   @work  │
│    Shopping   (1) │    [▸]  ! Write documentation       Due: Tomorrow       │
│                   │    [ ]    Update dependencies       Due: Fri 17         │
│  ───────────────  │    [✓]    Setup CI pipeline         Done: Yesterday     │
│                   │                                                         │
│  TAGS             │                                                         │
│  ────             │                                                         │
│    #urgent    (2) │                                                         │
│    #blocked   (1) │                                                         │
│    #meeting   (3) │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
├───────────────────┴─────────────────────────────────────────────────────────┤
│ [a]dd  [e]dit  [d]elete  [Space] toggle  [/] search  [?] help   Filter: All │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Legend:**
- `▸` = Selected item (highlighted row)
- `›●` = Selected project in sidebar
- `!!` = Urgent, `!` = High priority
- `[ ]` = Pending, `[▸]` = In Progress, `[✓]` = Completed

---

## 2. Task Detail View

Shown when pressing `Enter` on a task.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Task Details                                                    [Esc] Close │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Title:       Review PR #142                                                │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  Status:      ○ Pending   ◉ In Progress   ○ Completed                       │
│                                                                             │
│  Priority:    ○ Low   ○ Medium   ○ High   ◉ Urgent                          │
│                                                                             │
│  Due Date:    2025-01-13 (Today)                                            │
│                                                                             │
│  Project:     Work                                                          │
│                                                                             │
│  Tags:        #urgent  #code-review                                         │
│                                                                             │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Description:                                                               │
│                                                                             │
│  Review the authentication changes in PR #142. Check for:                   │
│  - Security vulnerabilities                                                 │
│  - Code style compliance                                                    │
│  - Test coverage                                                            │
│                                                                             │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Created:  2025-01-10 09:30    Updated:  2025-01-12 14:20                   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [e] Edit   [Space] Toggle Status   [p] Priority   [d] Delete   [Esc] Back   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Add/Edit Task Dialog

Modal dialog for creating or editing tasks.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado                                                                      │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │┌───────────────────────────────────────────────────────┐│
│  PROJECTS         ││ Add New Task                            [Esc] Cancel ││
│  ─────────        │├───────────────────────────────────────────────────────┤│
│  ● Inbox      (3) ││                                                       ││
│ ›● Work       (5) ││  Title:                                               ││
│    Personal   (2) ││  ┌─────────────────────────────────────────────────┐  ││
│    Shopping   (1) ││  │ Write unit tests for auth module█               │  ││
│                   ││  └─────────────────────────────────────────────────┘  ││
│  ───────────────  ││                                                       ││
│                   ││  Due Date:        Priority:       Project:            ││
│  TAGS             ││  ┌────────────┐   ┌───────────┐   ┌───────────┐       ││
│  ────             ││  │ 2025-01-15 │   │ ▾ High    │   │ ▾ Work    │       ││
│    #urgent    (2) ││  └────────────┘   └───────────┘   └───────────┘       ││
│    #blocked   (1) ││                                                       ││
│    #meeting   (3) ││  Tags:                                                ││
│                   ││  ┌─────────────────────────────────────────────────┐  ││
│                   ││  │ #testing #auth                                  │  ││
│                   ││  └─────────────────────────────────────────────────┘  ││
│                   ││                                                       ││
│                   ││  Description:                                         ││
│                   ││  ┌─────────────────────────────────────────────────┐  ││
│                   ││  │ Cover edge cases for login and token refresh   │  ││
│                   ││  │ █                                               │  ││
│                   ││  └─────────────────────────────────────────────────┘  ││
│                   ││                                                       ││
│                   │├───────────────────────────────────────────────────────┤│
│                   ││        [Tab] Next Field    [Enter] Save    [Esc] Cancel│
│                   │└───────────────────────────────────────────────────────┘│
├───────────────────┴─────────────────────────────────────────────────────────┤
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Calendar View

Monthly calendar with task indicators (press `c`).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Calendar                                              [←] Prev  [→] Next    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                            January 2025                                     │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│    Mon       Tue       Wed       Thu       Fri       Sat       Sun         │
│  ┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐   │
│  │         │         │    1    │    2    │    3    │    4    │    5    │   │
│  │         │         │         │         │    ●    │         │         │   │
│  ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤   │
│  │    6    │    7    │    8    │    9    │   10    │   11    │   12    │   │
│  │         │    ●    │         │         │   ●●    │         │         │   │
│  ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤   │
│  │▸  13    │   14    │   15    │   16    │   17    │   18    │   19    │   │
│  │  ●●●!   │    ●    │   ●●    │         │    ●    │         │         │   │
│  ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤   │
│  │   20    │   21    │   22    │   23    │   24    │   25    │   26    │   │
│  │         │    ●    │         │         │         │         │         │   │
│  ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤   │
│  │   27    │   28    │   29    │   30    │   31    │         │         │   │
│  │         │         │         │    ●    │         │         │         │   │
│  └─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘   │
│                                                                             │
│  Today: 3 tasks  ●●● (2 urgent!)                                            │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • !! Review PR #142                                                        │
│  • !! Fix login bug                                                         │
│  •  ! Write documentation                                                   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [Enter] View Day   [←/→] Change Month   [t] Today   [Esc] Back to Tasks     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Legend:**
- `●` = Task due on this day
- `!` = Has urgent/high priority tasks
- `▸` = Today (highlighted)

---

## 5. Search View

Quick search with live filtering (press `/`).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Search Tasks                                                    [Esc] Close │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Search: ┌────────────────────────────────────────────────────────────────┐ │
│          │ auth█                                                          │ │
│          └────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  Results (3 matches):                                                       │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  ▸ [ ] !! Fix login bug                       Due: Today        @Work      │
│         "...fix the auth token refresh..."                                  │
│                                                                             │
│    [ ]    Write unit tests for auth module    Due: Jan 15       @Work      │
│         "...cover auth edge cases..."                                       │
│                                                                             │
│    [✓]    Setup authentication middleware     Done: Jan 8       @Work      │
│         "...implement JWT auth flow..."                                     │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ [↑/↓] Navigate   [Enter] Open Task   [Esc] Cancel                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Help Overlay

Quick reference shown when pressing `?`.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado                                                                      │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │┌───────────────────────────────────────────────────────┐│
│  PROJECTS         ││ Keyboard Shortcuts                      [?] Close    ││
│  ─────────        │├───────────────────────────────────────────────────────┤│
│  ● Inbox      (3) ││                                                       ││
│ ›● Work       (5) ││  NAVIGATION           TASK ACTIONS                    ││
│    Personal   (2) ││  ───────────          ────────────                    ││
│    Shopping   (1) ││  j / ↓    Move down   a       Add task                ││
│                   ││  k / ↑    Move up     e       Edit task               ││
│  ───────────────  ││  g        First item  d       Delete task             ││
│                   ││  G        Last item   Space   Toggle done             ││
│  TAGS             ││  Tab      Switch pane p       Cycle priority          ││
│  ────             ││  h / l    Collapse    t       Edit tags               ││
│    #urgent    (2) ││                       m       Move to project         ││
│    #blocked   (1) ││  VIEWS                                                ││
│    #meeting   (3) ││  ─────                FILTERS                         ││
│                   ││  /        Search      ───────                         ││
│                   ││  c        Calendar    1-4     By priority             ││
│                   ││  ?        This help   T       Today only              ││
│                   ││                       W       This week               ││
│                   ││  GENERAL              A       Show archived           ││
│                   ││  ───────              f       Filter menu             ││
│                   ││  q        Quit        s       Sort menu               ││
│                   ││  Esc      Back/Close                                  ││
│                   ││  r        Refresh                                     ││
│                   │└───────────────────────────────────────────────────────┘│
├───────────────────┴─────────────────────────────────────────────────────────┤
│                                Press any key to close                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Filter Menu

Dropdown menu for filtering tasks (press `f`).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado                                              [!2 overdue] [Today: 4] │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │                                                         │
│  PROJECTS         │  TASKS                                                  │
│  ─────────        │  ────── ┌─────────────────────────┐                     │
│  ● Inbox      (3) │         │ Filter by               │                     │
│ ›● Work       (5) │  ▸ [ ] !├─────────────────────────┤           @work     │
│    Personal   (2) │    [ ] !│ ▸ All Tasks         (11)│           @work     │
│    Shopping   (1) │    [▸]  │   Pending            (6)│  Tomorrow           │
│                   │    [ ]  │   In Progress        (2)│  Fri 17             │
│  ───────────────  │    [✓]  │   Completed          (3)│  Yesterday          │
│                   │         │   ─────────────────────│                      │
│  TAGS             │         │   Due Today          (3)│                     │
│  ────             │         │   Due This Week      (5)│                     │
│    #urgent    (2) │         │   Overdue            (2)│                     │
│    #blocked   (1) │         │   No Due Date        (1)│                     │
│    #meeting   (3) │         │   ─────────────────────│                      │
│                   │         │   Urgent Priority    (2)│                     │
│                   │         │   High Priority      (3)│                     │
│                   │         └─────────────────────────┘                     │
│                   │                                                         │
├───────────────────┴─────────────────────────────────────────────────────────┤
│ [↑/↓] Select   [Enter] Apply   [Esc] Cancel                   Filter: All   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Sort Menu

Dropdown menu for sorting tasks (press `s`).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado                                              [!2 overdue] [Today: 4] │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │                                                         │
│  PROJECTS         │  TASKS                                                  │
│  ─────────        │  ────── ┌─────────────────────────┐                     │
│  ● Inbox      (3) │         │ Sort by                 │                     │
│ ›● Work       (5) │  ▸ [ ] !├─────────────────────────┤           @work     │
│    Personal   (2) │    [ ] !│ ▸ Due Date (asc)      ✓ │           @work     │
│    Shopping   (1) │    [▸]  │   Due Date (desc)       │  Tomorrow           │
│                   │    [ ]  │   Priority (high first) │  Fri 17             │
│  ───────────────  │    [✓]  │   Priority (low first)  │  Yesterday          │
│                   │         │   Created (newest)      │                     │
│  TAGS             │         │   Created (oldest)      │                     │
│  ────             │         │   Alphabetical (A-Z)    │                     │
│    #urgent    (2) │         │   Alphabetical (Z-A)    │                     │
│    #blocked   (1) │         └─────────────────────────┘                     │
│    #meeting   (3) │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
├───────────────────┴─────────────────────────────────────────────────────────┤
│ [↑/↓] Select   [Enter] Apply   [Esc] Cancel                   Filter: All   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 9. Empty State

When no tasks exist or filter returns nothing.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado v0.1.0                                                               │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │                                                         │
│  PROJECTS         │                                                         │
│  ─────────        │                                                         │
│ ›● Inbox      (0) │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │              No tasks yet!                              │
│                   │                                                         │
│                   │         Press [a] to add your first task                │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
│                   │                                                         │
├───────────────────┴─────────────────────────────────────────────────────────┤
│ [a]dd  [?] help                                                 Filter: All │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 10. Notification Banner

Shown at top when reminders trigger.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ⚠ OVERDUE: "Review PR #142" was due 2 hours ago                  [x] Dismiss│
├─────────────────────────────────────────────────────────────────────────────┤
│ Ratado v0.1.0                                       [!2 overdue] [Today: 4] │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │                                                         │
│  ...              │  ...                                                    │
```

---

## 11. Delete Confirmation

Modal confirmation before deleting.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ratado                                                                      │
├───────────────────┬─────────────────────────────────────────────────────────┤
│                   │                                                         │
│  PROJECTS         │  TASKS                                                  │
│  ─────────        │  ───── ┌───────────────────────────────────────────┐    │
│  ● Inbox      (3) │        │                                           │    │
│ ›● Work       (5) │  ▸ [ ] │   Delete Task?                            │    │
│    Personal   (2) │    [ ] │                                           │    │
│    Shopping   (1) │    [▸] │   "Review PR #142"                        │    │
│                   │    [ ] │                                           │    │
│  ───────────────  │    [✓] │   This action cannot be undone.           │    │
│                   │        │                                           │    │
│  TAGS             │        │         [y] Yes, Delete    [n] Cancel     │    │
│  ────             │        │                                           │    │
│    #urgent    (2) │        └───────────────────────────────────────────┘    │
│    #blocked   (1) │                                                         │
│    #meeting   (3) │                                                         │
│                   │                                                         │
├───────────────────┴─────────────────────────────────────────────────────────┤
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 12. Compact/Narrow Terminal

Responsive layout for smaller terminals (< 80 columns).

```
┌───────────────────────────────────────────┐
│ Ratado                    [!2] [Today: 4] │
├───────────────────────────────────────────┤
│ Projects: Work (5)              [Tab] ←→  │
├───────────────────────────────────────────┤
│                                           │
│ ▸ [ ] !! Review PR #142       Today       │
│   [ ] !! Fix login bug        Today       │
│   [▸]  ! Write documentation  Tomorrow    │
│   [ ]    Update dependencies  Fri 17      │
│   [✓]    Setup CI pipeline    Done        │
│                                           │
│                                           │
│                                           │
├───────────────────────────────────────────┤
│ [a]dd [d]el [Space]toggle [/]search [?]   │
└───────────────────────────────────────────┘
```

---

## 13. Debug Logs View (F12)

tui-logger widget for development debugging. Toggle with `F12`.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Debug Logs                                                      [Esc] Close │
├─────────────────────┬───────────────────────────────────────────────────────┤
│                     │                                                       │
│  TARGETS            │  LOG MESSAGES                                         │
│  ───────            │  ────────────                                         │
│                     │                                                       │
│ ▸[DEBUG] ratado     │  14:23:01 DEBUG ratado::storage                       │
│  [INFO ] storage    │    Database connection established                    │
│  [WARN ] handlers   │                                                       │
│  [ERROR] ui         │  14:23:01 INFO  ratado::app                           │
│                     │    Application initialized                            │
│  ─────────────────  │                                                       │
│                     │  14:23:05 DEBUG ratado::handlers                      │
│  Level: DEBUG       │    Key pressed: j (NavigateDown)                      │
│                     │                                                       │
│  [h] hide targets   │  14:23:05 DEBUG ratado::storage                       │
│  [f] focus target   │    Query: SELECT * FROM tasks WHERE status = ?        │
│  [↑↓] navigate      │    Duration: 1.2ms                                    │
│  [←→] change level  │                                                       │
│  [PgUp/Dn] scroll   │  14:23:06 WARN  ratado::notifications                 │
│                     │    Task "Review PR" is overdue by 2 hours             │
│                     │                                                       │
│                     │  14:23:10 INFO  ratado::handlers                      │
│                     │    Task created: "New task" (id: abc-123)             │
│                     │                                                       │
├─────────────────────┴───────────────────────────────────────────────────────┤
│ [h] targets  [f] focus  [←→] level  [PgUp/Dn] scroll            [Esc] Close │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Target Selector (left panel):**
- Shows log sources/modules
- Brackets show current display level: `[DEBUG]`, `[INFO]`, `[WARN]`, `[ERROR]`
- Use `←/→` to filter what levels are shown per target

**Log Messages (right panel):**
- Scrollable history of log events
- Color-coded by level (DEBUG=cyan, INFO=green, WARN=yellow, ERROR=red)
- Shows timestamp, level, target, and message

---

## Color Reference

| Element | Color | ANSI Code |
|---------|-------|-----------|
| Normal text | White | Default |
| Selected row | White on Blue | `\x1b[44m` |
| Urgent `!!` | Red Bold | `\x1b[1;31m` |
| High `!` | Yellow | `\x1b[33m` |
| Overdue | Red | `\x1b[31m` |
| Due Today | Yellow | `\x1b[33m` |
| Completed | Dim Gray | `\x1b[2m` |
| Project selected | Cyan | `\x1b[36m` |
| Tags | Magenta | `\x1b[35m` |
| Borders | Gray | `\x1b[90m` |
| Success msg | Green | `\x1b[32m` |
| Error msg | Red Bold | `\x1b[1;31m` |

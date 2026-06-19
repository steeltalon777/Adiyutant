# Contract Examples

Golden JSON snapshots for Android-Core API contract.
Each file represents a stable DTO shape that Android clients can rely on.

## Stability Guarantee

- Field additions are backward-compatible.
- Field renames are breaking changes.
- New optional fields use empty string `""` or sensible defaults.

## Files

| File | DTO | Screen |
|------|-----|--------|
| startup-state.json | StartupViewDto | Startup |
| today-dashboard.json | TodayViewDto | Today Dashboard |
| current-activity.json | CurrentActivityDto | Current Activity |
| plan-item-lifecycle.json | PlanItemDto × 3 stages | Plan Screen |
| waiting-review.json | Vec\<WaitingTaskDto\> | Waiting Review |
| checklist-run.json | ChecklistTemplateDto + ChecklistRunDto | Checklist Run |
| journal-feed.json | Vec\<JournalEntryDto\> | Journal |

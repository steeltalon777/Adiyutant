# Architecture Review — Phase 8.4A Portable Bundle Contract

**Date:** 2026-06-22  
**Reviewer:** Architect

## Verdict

Approved with conditions.

The plan is acceptable because it keeps the transfer format separate from Project/Roadmap domain work, requires one shared preview/apply planner, treats zip input as untrusted, and adds import history plus all-or-nothing apply. No blockers remain in the TZ after the conditions below are included as implementation requirements.

## 🔴 Blockers

None.

## 🟡 Warnings

### 1. Zip metadata support must be validated

- **Checklist item:** Security — untrusted archive input.
- **Issue:** The TZ requires rejecting symlinks/hardlinks, path traversal, unsafe extensions, and size bombs. Some zip APIs do not expose all Unix mode metadata consistently.
- **Impact:** A weak archive parser could allow path traversal or hidden filesystem objects during extraction/parsing.
- **Recommendation:** Validate the selected zip crate before implementation. If symlink/hardlink detection is not reliable, stop and report a dependency blocker instead of weakening the requirement.

### 2. Existing records lack stable slugs

- **Checklist item:** Data & State — source of truth and merge identity.
- **Issue:** Current `ContextDocument` and `ChecklistTemplate` primarily use UUID/title/doc_type. Append/merge semantics require stable slugs without breaking existing rows.
- **Impact:** Merge could create duplicates like repeated morning checklists or overwrite the wrong context document.
- **Recommendation:** Add nullable slug/source fields with backward-compatible migration and tests for old rows. Unique indexes must tolerate null existing values.

### 3. Preview/apply drift is a high-risk failure mode

- **Checklist item:** Failure Modes — partial/incorrect apply.
- **Issue:** If preview and apply compute actions separately, apply can write a different set of changes from what the user approved.
- **Impact:** User trust is broken; import preview becomes misleading.
- **Recommendation:** Use one internal planner. Apply must consume or recompute the same plan deterministically and test action parity.

### 4. Transaction boundary must live below service-level loops

- **Checklist item:** Data & State — transactionality.
- **Issue:** The current Store trait has composite operations but no generic public transaction. A service-level loop of individual writes cannot guarantee rollback.
- **Impact:** Failed checklist import could leave context/rules partially written.
- **Recommendation:** Add a Store-level import composite or bounded transaction seam. Include rollback tests.

### 5. Unknown field warnings require custom validation path

- **Checklist item:** Operability — diagnostics and user feedback.
- **Issue:** Standard strict serde deserialization usually errors on unknown fields or ignores them, but this plan needs warning-and-ignore for optional fields.
- **Impact:** Assistant-generated bundles could either fail too aggressively or hide useful diagnostics.
- **Recommendation:** Parse YAML to an inspectable value first or otherwise preserve unknown-key diagnostics before converting to typed structs.

## 🔵 Notes

### 1. Export overwrite behavior can be conservative

- **Checklist item:** Operability — rollback/deploy safety.
- **Issue:** Exporting into existing directories can be ambiguous.
- **Recommendation:** Prefer failing on unsafe existing targets unless an explicit overwrite behavior is implemented and tested.

### 2. Rule Engine v2 remains separate

- **Checklist item:** Complexity — simplest solution.
- **Issue:** Mapping human rules into executable suggestions is tempting but outside the import contract.
- **Recommendation:** Keep rules as context in 8.4A.

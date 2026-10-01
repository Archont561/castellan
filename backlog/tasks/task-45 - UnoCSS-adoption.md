---
id: TASK-45
title: UnoCSS adoption
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - ux
  - infrastructure
dependencies: []
priority: low
ordinal: 12800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The styling item deferred earlier and never tracked: UnoCSS as a workspace devDependency with one shared preset consumed by packages/ui and the SvelteKit apps - atomic utilities without shipping a component framework's CSS. Component tests and Storybook must stay green through the migration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 UnoCSS is a workspace devDependency with one shared preset consumed by packages/ui and the SvelteKit apps
- [ ] #2 Existing components render identically (component tests and Storybook unchanged green)
- [ ] #3 The extract pipeline runs through the vite plugin without breaking WXT or SvelteKit builds
<!-- AC:END -->

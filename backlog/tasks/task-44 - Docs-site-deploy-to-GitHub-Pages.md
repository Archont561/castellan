---
id: TASK-44
title: Docs site deploy to GitHub Pages
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - infrastructure
dependencies: []
priority: low
ordinal: 11800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The deploy the astro config already anticipates: a docs workflow adapted from pixi-sandbox's docs.yml with bun instead of pixi - builds apps/docs and publishes to Pages. site and base are already set in astro.config.mjs so the build passes through unchanged.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A docs workflow builds apps/docs and publishes to GitHub Pages (bun not pixi)
- [ ] #2 The workflow runs only on the default branch and is actionlint-clean
- [ ] #3 The published URL matches the site and base already configured in astro.config.mjs
<!-- AC:END -->

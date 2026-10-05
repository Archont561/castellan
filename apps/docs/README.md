# Castellan documentation site

This is the public-facing documentation site, built with Astro and Starlight.
It turns the repository's durable architecture, security, protocol, testing,
and roadmap material into a navigable handbook; it does not replace the source
of truth in code, generated protocol files, or the delivery backlog.

## Run and build

Run these commands from the repository root through Pixi:

```console
$ pixi run dev-docs
$ pixi run docs-build
$ pixi run bun run --cwd apps/docs preview
```

## Layout

| Path | Responsibility |
| --- | --- |
| `src/content/docs/` | MDX documentation pages |
| `astro.config.mjs` | Starlight configuration and hand-authored sidebar order |
| `src/assets/` | Site-local images and assets |

When adding a page, add the MDX file under `src/content/docs/` and its sidebar
entry in `astro.config.mjs`. The sidebar is intentionally explicit: a generated
file-tree navigation documents storage order, not the reading order a new
contributor needs.

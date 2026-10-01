import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import icon from "astro-icon";

// ---------------------------------------------------------------------------
// Copied from pixi-sandbox's docs app (the template this site was cut from),
// adjusted twice:
//
// 1. No version-substitution rig. pixi-sandbox pipes its Markdown through
//    @astrojs/markdown-satteri so an mdast plugin can replace __VERSION__
//    tokens with the version read from the workspace Cargo.toml — its pages
//    cite release tags constantly. Castellan's pages describe an
//    unreleased project and cite no versions, so the machinery would be
//    cargo cult today. When the first versioned page appears, copy the
//    whole rig from pixi-sandbox's docs/astro.config.mjs.
// 2. No deploy workflow yet. `site`/`base` are set so the config is already
//    correct for GitHub Pages; the workflow that publishes it is
//    pixi-sandbox's .github/workflows/docs.yml (adapted: bun instead of
//    pixi), added the day the site goes public.
//
// The rest is the template's shape: Starlight, astro-icon with four
// iconify sets, expressive code with matched light/dark themes, an edit
// link back to the repository, and a hand-written sidebar — because a
// generated sidebar documents the file tree, not the reading order.
// ---------------------------------------------------------------------------

export default defineConfig({
  site: "https://archont561.github.io",
  base: "/castellan",
  integrations: [
    icon({
      include: {
        lucide: ["*"],
        mdi: ["*"],
        "simple-icons": ["*"],
        tabler: ["*"]
      }
    }),
    starlight({
      title: "Castellan",
      description:
        "The local-first password manager: one KDBX vault, three faces, a soft security key, and a LAN device mesh — no cloud anywhere",
      logo: {
        light: "./src/assets/logo-light.svg",
        dark: "./src/assets/logo-dark.svg",
        replacesTitle: false
      },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/Archont561/castellan" }
      ],
      editLink: {
        baseUrl: "https://github.com/Archont561/castellan/edit/main/docs/"
      },
      customCss: ["./src/styles/custom.css"],
      sidebar: [
        {
          label: "Start here",
          items: [
            { label: "Introduction", slug: "index" },
            { label: "Roadmap", slug: "roadmap" }
          ]
        },
        {
          label: "Guides",
          items: [
            { label: "Development", slug: "development" },
            { label: "Testing", slug: "testing" }
          ]
        },
        {
          label: "Design",
          items: [
            { label: "Architecture", slug: "architecture" },
            { label: "The protocol", slug: "protocol" },
            { label: "The native channel", slug: "native-channel" },
            { label: "Security model", slug: "security" }
          ]
        },
        {
          label: "Reference",
          items: [{ label: "Repository", slug: "repository" }]
        }
      ],
      expressiveCode: {
        themes: ["github-dark", "github-light"]
      }
    })
  ]
});

/**
 * The shared UnoCSS configuration: one source of truth for four consumers
 * (desktop, mobile, the extension popup, and the component library's own
 * Storybook/component-test harnesses), next to `libPreset` and `e2ePreset`
 * for the same reason — the choices that must not drift between faces live
 * in one place, and each consumer's `uno.config.ts` is two lines.
 *
 * Why a preset rather than utilities-as-written: the faces share a *look*
 * and disagree about *metrics* (a desktop row is denser than a touch
 * target), so the look is a shortcut here and the sizing stays in the
 * markup. A component that wants `c-action` gets the castellan button in
 * every face; a face that wants it full-width says so itself.
 *
 * The tokens stay CSS custom properties even though UnoCSS could inline the
 * hex values: `app.css` used to carry them with the comment "faces theme by
 * overriding these", and that is still true — overriding `--accent` on a
 * subtree re-themes every utility built from it, which inlined colors
 * cannot do. The cost is that `/50` opacity shorthand does not work on
 * token colors (no rgb channels to split); the currentColor tints below
 * cover every case this UI actually has.
 */

import type { Preset, UserConfig } from "unocss";
import { defineConfig, presetWind3 } from "unocss";

/** The palette, as it was in both faces' `app.css` before this file existed. */
const tokens = {
  bg: "#14161a",
  fg: "#e8eaf0",
  accent: "#d4a24e",
  muted: "#8a90a0",
  surface: "#1d2026",
  border: "#2a2e37",
  // Two states that were inline hex in three components; naming them is the
  // whole reason a token file exists.
  danger: "#e06c75",
  ok: "#4caf7d"
} as const;

const fontSans = 'system-ui, -apple-system, "Segoe UI", sans-serif';

/**
 * A translucent shade of whatever the element's text color is. Every
 * hairline and secondary label in the UI was already
 * `color-mix(in oklab, currentColor N%, transparent)` by hand — as a rule it
 * keeps that exact output (oklab, not sRGB: the mix stays perceptually even
 * on both the dark app surface and Storybook's light one) and stops the
 * percentages from drifting apart across components.
 */
const tint = (percent: string): string =>
  `color-mix(in oklab, currentColor ${percent}%, transparent)`;

const vars = (): string =>
  [
    ...Object.entries(tokens).map(([name, value]) => `--${name}: ${value};`),
    `--font-sans: ${fontSans};`
  ].join("\n  ");

/** Options a consumer can disagree about. Everything else is not negotiable. */
export interface UnoPresetOptions {
  /**
   * Paint the page shell (`html`/`body`: the dark surface, the full height,
   * the font). True for the two SvelteKit faces, which own their window.
   * False for the extension popup and for Storybook/component tests, which
   * render *into* a surface somebody else owns — a popup that paints a
   * full-bleed background is a popup that fights the browser's chrome.
   */
  shell?: boolean;
}

/**
 * The castellan preset: tokens, the currentColor tints, and the handful of
 * recurring component looks. Exported separately from `unoPreset` so a
 * consumer with its own base preset can still get the house style.
 */
export function presetCastellan(options: UnoPresetOptions = {}): Preset {
  const { shell = true } = options;

  return {
    name: "@castellan/uno-preset",
    theme: {
      colors: Object.fromEntries(Object.keys(tokens).map((name) => [name, `var(--${name})`])),
      fontFamily: { sans: "var(--font-sans)" }
    },
    rules: [
      [/^bg-tint-(\d{1,3})$/, ([, p = "0"]) => ({ "background-color": tint(p) })],
      [/^text-tint-(\d{1,3})$/, ([, p = "0"]) => ({ color: tint(p) })],
      [/^border-tint-(\d{1,3})$/, ([, p = "0"]) => ({ "border-color": tint(p) })],
      [/^stroke-tint-(\d{1,3})$/, ([, p = "0"]) => ({ stroke: tint(p) })],
      // `font: inherit` is a shorthand no utility preset models, and it is
      // what keeps a <button> from falling back to the UA's 13px Arial.
      ["font-inherit", { font: "inherit" }]
    ],
    shortcuts: {
      // A vault row: the grid is the contract (title grows, badges do not),
      // and the button reset is what makes a whole row clickable without
      // looking like a button.
      "c-entry-row":
        "grid grid-cols-[1fr_auto_auto] items-baseline gap-3 w-full px-[0.8rem] py-[0.6rem] text-left text-inherit font-inherit bg-transparent b-0 rounded-8px cursor-pointer hover:bg-tint-8",
      // A capability dot ("2FA", the passkey glyph): sized in `em` on
      // purpose, so it shrinks with whatever row it lands in.
      "c-badge": "text-[0.7em] px-[0.4rem] py-[0.1rem] rounded-full b-1 b-solid border-tint-30",
      // The sunken surface: generated passphrases, codes, anything the user
      // reads back rather than types.
      "c-field": "block bg-surface b-1 b-solid border-border rounded-8px",
      // The one affirmative button. Metrics stay with the face — padding,
      // width, and the corner radius, which really is a metric here: the
      // desktop button is 8px and the thumb-sized mobile one is 10px. A
      // radius in the shortcut would collide with the face's own
      // `rounded-*` and leave the winner to CSS source order.
      "c-action": "bg-accent text-bg font-600 b-1 b-solid border-border cursor-pointer",
      // Section headings: the quiet uppercase label above a list.
      "c-section-title": "text-muted uppercase tracking-[0.08em]",
      // The TOTP countdown: a track and the arc that empties against it.
      // `-rotate-90` moves 12 o'clock to the start of the arc.
      "c-ring-track": "fill-none stroke-tint-20 [stroke-width:2.5]",
      // `[transition-property:…]`, not `transition-[…]`: presetWind3 has no
      // arbitrary-value form of the transition shorthand, and an unmatched
      // utility inside a shortcut is a *warning*, not an error — the ring
      // would have silently transitioned `all` properties at 1s instead of
      // just the dash offset.
      "c-ring-progress":
        "fill-none stroke-current origin-center -rotate-90 [stroke-width:2.5] [transition-property:stroke-dashoffset] duration-1000 ease-linear"
    },
    preflights: [
      {
        // Tokens always: a consumer that opts out of the shell still builds
        // utilities that resolve to these variables.
        getCSS: () => `:root {\n  ${vars()}\n}`
      },
      ...(shell
        ? [
            {
              getCSS: () => `html, body {
  margin: 0;
  height: 100%;
  background: var(--bg);
  color: var(--fg);
  font-family: var(--font-sans);
}
button {
  font: inherit;
}`
            }
          ]
        : [])
    ]
  };
}

/**
 * The full shared config. A consumer's `uno.config.ts` is
 * `defineConfig(unoPreset())` and nothing else.
 */
export function unoPreset(options: UnoPresetOptions = {}): UserConfig {
  return defineConfig({
    presets: [presetWind3(), presetCastellan(options)],
    // Prose, not markup. The extractor tokenises whole files, comments
    // included, so the words "ring" (TotpRing documents itself) and "grid"
    // ("the desktop grid") each matched a real presetWind3 utility and
    // shipped a rule no element references — 300 bytes of box-shadow
    // variables in every face. Blocking the bare tokens leaves `ring-2`,
    // `grid-cols-*` and the shortcuts that expand to `display: grid`
    // untouched; if a design ever wants a plain `grid` class, it arrives
    // here as a deliberate deletion.
    blocklist: ["ring", "grid"],
    content: {
      pipeline: {
        include: [
          // UnoCSS's own default set: templates, not every module.
          /\.(vue|svelte|[jt]sx|mdx?|astro|elm|php|phtml|html)($|\?)/,
          // @castellan/ui is consumed as *source* through a workspace
          // symlink, so its components are part of the consuming app's
          // module graph and must be part of the consuming app's extraction
          // pass — otherwise every shared component ships with its classes
          // unbuilt and renders unstyled in the app while looking perfect in
          // Storybook. Matched by path rather than by resolved location so
          // it holds whether vite hands us the real path (the default) or
          // the symlinked one under node_modules (`preserveSymlinks`).
          /[\\/]packages[\\/]ui[\\/]src[\\/].*\.(svelte|ts)($|\?)/,
          /[\\/]node_modules[\\/]@castellan[\\/]ui[\\/]src[\\/].*\.(svelte|ts)($|\?)/
        ],
        exclude: [
          // Everything in node_modules except our own workspace packages,
          // whose source is the point of the include above.
          /[\\/]node_modules[\\/](?!@castellan[\\/])/,
          /[\\/]\.(git|svelte-kit|wxt|output|turbo|pixi)[\\/]/,
          /\.(css|postcss|sass|scss|less|stylus|styl)($|\?)/
        ]
      }
    }
  });
}

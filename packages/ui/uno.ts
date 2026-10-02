/**
 * The component looks, as UnoCSS shortcuts — the design-system half of the
 * styling setup.
 *
 * Why here and not in `@castellan/utils/uno`: utils owns the *foundation*
 * (the tokens, the currentColor tints, the page shell, the extraction
 * pipeline) because everything that renders needs it, including the
 * extension popup, which deliberately has no dependency on this package.
 * The shortcuts are something else — they are the shared appearance of the
 * castellan UI, they change when a component changes, and the components
 * they describe live in `src/`. Keeping them next to the components means a
 * single pull request can move a look and its markup together, and a face
 * that opts into `@castellan/ui` opts into the looks by the same import.
 *
 * Three of these (`c-field`, `c-action`, `c-section-title`) have no Svelte
 * component yet: they are looks the desktop and mobile faces both write by
 * hand. They live here anyway — this file is the house style, not the
 * export list — and the day one of them grows a component, the look is
 * already in the right package.
 *
 * The rule that keeps the two halves from fighting: a shortcut carries the
 * *look* that must not drift between faces; metrics (padding, width, radius,
 * font size) stay in the face's markup, and no property may appear in both.
 *
 * This file sits at the package root on purpose. `src/**` is in the shared
 * extraction pipeline, so a shortcut table under `src/` would be scanned as
 * if it were markup and every utility named in it would ship as a standalone
 * rule in every face.
 */

import type { Preset } from "unocss";

/**
 * The castellan component looks. Layered on top of `presetCastellan` by the
 * consumers that render these components — the two faces and this package's
 * own Storybook and component-test harnesses.
 */
export function presetUi(): Preset {
  return {
    name: "@castellan/ui-preset",
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
    }
  };
}

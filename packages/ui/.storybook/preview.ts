import type { Preview } from "@storybook/svelte-vite";
// The same generated utilities the apps build, from the same shared
// config: a story that renders differently from the face would be worse
// than no story at all.
import "virtual:uno.css";

/**
 * Deliberately almost empty: the components are presentational, carry
 * their own styles, and expect to sit on the apps' dark surface — the
 * stories should show them as the apps see them, not inside Storybook's
 * default chrome.
 */
const preview: Preview = {
  parameters: {
    backgrounds: { default: "castellan" }
  }
};

export default preview;

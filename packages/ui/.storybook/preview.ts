import type { Preview } from "@storybook/svelte-vite";

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

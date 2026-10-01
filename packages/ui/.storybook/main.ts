import { defineMain } from "@storybook/svelte-vite/node";

/**
 * Storybook for the shared component library. The components are pure
 * Svelte with no Kit runtime, so the plain svelte-vite framework fits
 * without pulling an app shell in; stories are `.stories.svelte` files
 * next to their components (Svelte CSF), which keeps props and stories
 * in one typed place — svelte-check covers them like any other file.
 */
export default defineMain({
  framework: "@storybook/svelte-vite",
  stories: ["../src/**/*.stories.svelte"],
  addons: ["@storybook/addon-svelte-csf"]
});

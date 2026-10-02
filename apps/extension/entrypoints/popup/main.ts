// The generated utilities for this entrypoint. WXT builds each entrypoint
// separately, so importing it here keeps the popup's CSS in the popup and
// out of the content script.
import "virtual:uno.css";
import { mount } from "svelte";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error('popup mount failed: no element with id "app"');
}

const app = mount(App, { target });

export default app;

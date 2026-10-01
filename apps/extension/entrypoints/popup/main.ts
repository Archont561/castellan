import { mount } from "svelte";
import App from "./App.svelte";

const target = document.getElementById("app");
if (!target) {
  throw new Error('popup mount failed: no element with id "app"');
}

const app = mount(App, { target });

export default app;

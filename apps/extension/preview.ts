import "virtual:uno.css";
import { mount } from "svelte";
import App from "./entrypoints/popup/App.svelte";

const target = document.getElementById("app");
if (!target) throw new Error('preview mount failed: no element with id "app"');

mount(App, { target, props: { preview: true } });

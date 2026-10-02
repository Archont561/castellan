// The component-testing entry point. The Playwright CT vite plugin appends
// its runtime (the component registry every test's `mount()` talks to) to
// this file at build time; the one thing it needs from us is the
// stylesheet, because the components carry UnoCSS utility classes now and
// a component mounted without them is not the component the app renders.
import "virtual:uno.css";

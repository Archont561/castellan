// The library's own harnesses (Storybook and the Playwright component
// tests) build the same utilities the apps do, from the same two pieces —
// the foundation in @castellan/utils and this package's own component
// looks — so a story and a face cannot disagree about what a component
// looks like. `shell: false`: a story renders *into* Storybook's canvas, it
// does not own the page.
import { unoPreset } from "@castellan/utils/uno";
import { presetUi } from "./uno";

export default unoPreset({ shell: false, presets: [presetUi()] });

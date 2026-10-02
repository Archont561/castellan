// Same shared config as the desktop face; the two differ in metrics (touch
// targets, full-width actions), which is markup, not configuration.
import { presetUi } from "@castellan/ui/uno";
import { unoPreset } from "@castellan/utils/uno";

export default unoPreset({ presets: [presetUi()] });

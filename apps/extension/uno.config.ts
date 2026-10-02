// `shell: false`: the popup is a surface the browser sizes and frames, so
// it must not paint `html`/`body` the way an app window does — it still
// gets the same tokens, tints and component looks.
import { unoPreset } from "@castellan/utils/uno";

export default unoPreset({ shell: false });

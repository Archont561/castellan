// `shell: false`: the popup is a surface the browser sizes and frames, so
// it must not paint `html`/`body` the way an app window does — it still
// gets the same tokens and tints. No `presetUi()`: the popup renders no
// castellan components and depends on no component library, which is the
// reason the `c-*` shortcuts live in @castellan/ui and not in the
// foundation.
import { unoPreset } from "@castellan/utils/uno";

export default unoPreset({ shell: false });

// Three lines, like every bunup config in this repo: the foundation (tokens,
// tints, page shell, extraction) comes from @castellan/utils, the component
// looks come from @castellan/ui next to the components themselves, and the
// app only says that it uses both. `shell` stays on — the desktop face owns
// its whole window.
import { presetUi } from "@castellan/ui/uno";
import { unoPreset } from "@castellan/utils/uno";

export default unoPreset({ presets: [presetUi()] });

---
id: doc-12
title: "Castellan UI System Specification"
type: specification
created_date: '2026-10-02 19:55'
updated_date: '2026-10-02 20:38'
tags:
  - ui
  - ux
  - svelte
  - accessibility
  - specification
---
# Specification — Castellan UI system

**Scope.** The component and layout contracts for Castellan's desktop and
mobile app UI, plus the smaller browser-extension surfaces where those patterns
must remain compatible. This document is the canonical component spec; delivery
state lives in tasks, and implementation guidance for agents lives in
`packages/ui/AGENTS.md`.

**Reference patterns.** The layout vocabulary comes from the password-manager
shape shared by Bitwarden, KeePassXC and KeePassium: searchable vault lists,
large field-local copy/reveal actions, visible lock state, list/detail on large
screens, native-feeling list/detail navigation on mobile, and compact
site-focused browser-extension surfaces. Those references are design inputs,
not assets to copy.

## §1 Principles

1. **Metadata in lists, secrets in details.** Lists show title, username/host,
   type/capability badges and status. Passwords, TOTP seeds, passkey material
   and recovery codes never appear in list props.
2. **Actions live beside the value they affect.** Copy username, reveal/copy
   password, copy TOTP, open URL, and copy recovery code are field-local
   affordances, not hidden in a global toolbar.
3. **Lock state is always visible.** Desktop, mobile and extension surfaces all
   show whether the vault is locked, unlocked, disconnected or has no database.
4. **Search is primary.** Desktop gets a top search/command affordance; mobile
   gets a search field below the header; extension popup prioritizes current
   site matches but still exposes search when unlocked.
5. **Large screens use list/detail.** Desktop/tablet should keep navigation,
   entries and the selected entry visible at once when width allows.
6. **Small screens use navigation stacks and sheets.** Mobile list → detail →
   edit is a stack. Secondary actions use bottom sheets or action sheets.
7. **Dangerous actions are explicit.** Move to trash is reversible; permanent
   delete requires confirmation and never shares styling with a primary action.
8. **Headless by default.** `@castellan/ui` wraps Bits UI, Floating UI,
   Svelte 5 drawer primitives and icons behind Castellan components. Apps do
   not style third-party primitives directly.
9. **Every consequential action has a receipt.** The UI exposes the local
   action log defined in `backlog/docs/specifications/action-log/doc-13 - Local-Action-Log-Specification.md`; components that perform copy, reveal, fill, sync, edit, import/export, settings or developer-tool actions must give the caller a place to record the durable receipt.

## §2 Shared behavior baseline

Every interactive component must satisfy this baseline unless its component
spec states a stricter rule.

- Use a native interactive element when one exists (`button`, `input`, `a`).
- Visible focus ring is required in keyboard modality.
- Enter/Space activates buttons and button-like rows.
- Escape closes menus, popovers, dialogs and sheets.
- Focus returns to the opener after a transient surface closes.
- Pointer targets are at least 44 px tall on mobile and at least 32 px tall in
  compact desktop density.
- Loading, empty, error, disabled and locked states must be representable
  without layout jump.
- Semantic/test hook classes stay first in class lists and carry no styling.
- Components render from props and callbacks; they do not import transports,
  generated clients, Tauri APIs, WXT globals, or vault crates.

## §3 Surface layouts

### §3.1 Desktop vault shell

Desktop uses a three-pane app shell once the window is wide enough. The entry
list may become a table later, but the default view is list/detail because the
most common task is acting on one selected item.

```text
┌───────────────────────────────────────────────────────────────────────────┐
│ Castellan        [ Search vault… / ⌘K ]           🔓 Unlocked   Settings │
├──────────────────┬───────────────────────────┬────────────────────────────┤
│ VaultSidebar     │ EntryList                 │ EntryDetail                │
│                  │                           │                            │
│ All entries      │ github.com                │ GitHub                Edit │
│ Favorites        │ ada@castellan.dev   2FA 🔑│ github.com                 │
│ Logins           │                           │                            │
│ TOTP             │ docs.example              │ Username              Copy │
│ Passkeys         │ writer@example       2FA  │ Password        Reveal Copy│
│ Recovery codes   │                           │ TOTP 123 456  21s    Copy │
│ SSH keys         │                           │ URL              Open Copy │
│ Audit            │                           │                            │
│ Trash            │                           │ Notes                      │
│                  │                           │ …                          │
└──────────────────┴───────────────────────────┴────────────────────────────┘
```

Expected behavior:

- `⌘K` / `Ctrl+K` opens `CommandPalette`.
- `/` or focusing the search field starts text search when not inside an input.
- Arrow keys move through `EntryList`; Enter opens/selects the row.
- Detail actions are available by keyboard and pointer.
- Locked state replaces list/detail with a single unlock prompt; disconnected
  state names the missing transport or app connection.

### §3.2 Mobile vault shell

Mobile uses a single-column navigation stack. Search and lock state are visible
before the list. Entry quick actions appear in a bottom sheet.

```text
┌──────────────────────────────┐
│ Castellan              🔓    │
│ [ Search vault…             ]│
├──────────────────────────────┤
│ GitHub                       │
│ ada@castellan.dev      2FA 🔑│
├──────────────────────────────┤
│ Email                        │
│ user@example.com        2FA  │
├──────────────────────────────┤
│ Bank                         │
│ personal               🔑    │
├──────────────────────────────┤
│ Vault      Generator Settings│
└──────────────────────────────┘
```

Expected behavior:

- Tapping a row navigates to `EntryDetail`.
- Long-press or row overflow opens `EntryActionsSheet`.
- Pull-to-refresh is allowed only when the face has a real refresh operation;
  do not fake refresh for local data.
- Bottom navigation never contains destructive actions.
- Safe-area insets are honored at top and bottom.

### §3.3 Mobile entry detail

```text
┌──────────────────────────────┐
│ ‹ GitHub                 ⋯   │
├──────────────────────────────┤
│ github.com                   │
│ ada@castellan.dev            │
│                              │
│ Username                     │
│ ada@castellan.dev       Copy │
│                              │
│ Password                     │
│ •••••••••••••   Reveal Copy │
│                              │
│ One-time password            │
│ 123 456        21s      Copy │
│                              │
│ Passkey                      │
│ github.com              View │
│                              │
│ Notes                        │
│ …                            │
└──────────────────────────────┘
```

Expected behavior:

- Back returns to the previous list state and selection.
- Overflow opens `EntryActionsSheet` with edit, favorite, move, trash and
  advanced actions.
- Field copy actions provide immediate feedback and do not reveal the secret.
- Password reveal is temporary by default and re-hides on lock, navigation away
  or explicit hide.

### §3.4 Browser extension popup

The extension popup is not a full vault manager. It is current-site-first and
connection-state-first.

```text
┌──────────────────────────────┐
│ Fob                 Connected│
├──────────────────────────────┤
│ Current site                 │
│ github.com                   │
│                              │
│ Matching entries             │
│ GitHub                       │
│ ada@castellan.dev            │
│ [Fill] [TOTP] [More]         │
│                              │
│ Vault                        │
│ Unlocked                     │
│                              │
│ [Open Castellan]             │
│ [Lock vault]                 │
└──────────────────────────────┘
```

Expected behavior:

- Shows disconnected, locked, no-database and unlocked states explicitly.
- Current origin is visible before any fill action.
- Fill requires user gesture.
- TOTP code request happens only through the protocol; seeds never enter the
  extension.
- Repair/open-app actions are visible when the native connection fails.

### §3.5 Injected autofill/passkey overlay

Injected page UI is owned by the extension, not `@castellan/ui`; this spec
keeps its behavior compatible with app components.

```text
input field ───────────────────────┐
                                    │
                         ┌──────────▼──────────┐
                         │ Castellan           │
                         │ GitHub              │
                         │ ada@castellan.dev   │
                         │ [Fill] [TOTP]       │
                         └─────────────────────┘
```

Expected behavior:

- Render in Shadow DOM.
- Position with Floating UI against the focused field or WebAuthn prompt
  anchor.
- Never use global page CSS.
- Close on Escape, blur outside, frame navigation or vault lock.
- Do not show suggestions until the app validates origin matches.


### §3.6 Sync and device mesh surfaces

Sync borrows LocalSend's successful mental model — a clear local device identity,
nearby peers, explicit incoming requests, per-item progress, and conservative
"quick save" settings — but tightens it for vault data. Castellan never has an
"accept from everyone" mode for secrets: convenience starts at paired devices
and every destructive/conflicting change stays reviewable.

Desktop sync view:

```text
┌──────────────────┬───────────────────────────────────────────────────────┐
│ VaultSidebar     │ Sync                                                  │
│                  │                                                       │
│ All entries      │ This device                                           │
│ Sync             │ ┌───────────────────────────────────────────────────┐ │
│ Connected        │ │ Warsaw laptop                         Discoverable │ │
│ browsers         │ │ Fingerprint: castle-river-copper-lantern          │ │
│ Settings         │ │ LAN only · no cloud relay                         │ │
│                  │ └───────────────────────────────────────────────────┘ │
│                  │                                                       │
│                  │ Nearby paired devices                                 │
│                  │ ┌───────────────────────────────────────────────────┐ │
│                  │ │ Pixel 9                         last sync 2m ago  │ │
│                  │ │ 3 local changes · 1 remote change        [Sync]   │ │
│                  │ └───────────────────────────────────────────────────┘ │
│                  │                                                       │
│                  │ Pending requests                                      │
│                  │ iPad wants to pair                         [Review]  │
└──────────────────┴───────────────────────────────────────────────────────┘
```

Mobile sync view:

```text
┌──────────────────────────────┐
│ Sync                    ⚙    │
├──────────────────────────────┤
│ This device                  │
│ Warsaw phone                 │
│ castle-river-copper-lantern  │
│ LAN only                     │
│                              │
│ Receive from paired devices  │
│ [ Ask ] [ Auto ]             │
│                              │
│ Nearby devices               │
│ Pixel tablet                 │
│ paired · 1 change       Sync │
│                              │
│ Pair a new device            │
│ [Show QR] [Scan QR]          │
├──────────────────────────────┤
│ Vault      Sync     Settings │
└──────────────────────────────┘
```

Incoming pairing/sync request:

```text
┌──────────────────────────────┐
│ Pixel tablet                 │
│ Android · nearby             │
│                              │
│ wants to pair with this vault│
│                              │
│ Fingerprint phrase           │
│ castle-river-copper-lantern  │
│                              │
│ 0 entries shared yet         │
│                              │
│ [Decline]           [Approve]│
└──────────────────────────────┘
```

Change review before applying:

```text
┌──────────────────────────────────────────────┐
│ Review sync from Pixel tablet                │
├──────────────────────────────────────────────┤
│ Added                                       2 │
│ Modified                                    5 │
│ Conflicts                                   1 │
│ Deleted                                     0 │
│                                              │
│ Conflicts                                    │
│ GitHub                                      │
│ local password changed · remote TOTP changed │
│ [Merge] [Keep local] [Use remote]            │
│                                              │
│ [Cancel]                         [Apply 7]   │
└──────────────────────────────────────────────┘
```

Progress panel:

```text
┌──────────────────────────────────────────────┐
│ Syncing with Pixel tablet              42%   │
├──────────────────────────────────────────────┤
│ ✓ Verified device fingerprint                │
│ ✓ Created copy-aside backup                  │
│ ▬ Receiving encrypted delta                  │
│ · Merging entries                            │
│ · Writing vault                              │
│                                              │
│ Total progress                               │
│ ███████████░░░░░░░░░░░                       │
│ [Advanced log]                    [Cancel]   │
└──────────────────────────────────────────────┘
```

Expected behavior:

- Discovery state is visible: off, scanning, discoverable, no devices,
  devices found, network blocked/VPN likely, and server error.
- Pairing requires a human-verifiable phrase or QR on both devices before trust
  is stored.
- Auto-accept applies only to already paired devices and defaults off. If it is
  enabled, destructive changes and conflicts still require review.
- Every sync that can write the vault shows the copy-aside backup step before
  transfer or merge progress can complete.
- Progress is both per-stage and total; cancel says whether it is safe before
  write, waiting after write, or impossible during an atomic commit.
- Sync history is a local audit trail: peer, time, counts, result, backup path
  where relevant. It never stores secret values.
- Sync actions also create general action-log receipts, so a user can see sync next to extension, CLI, clipboard and vault-edit activity in one Activity view.

## §4 Component inventory

### §4.1 Primitives to create in `packages/ui/src/primitives/`

| Component | Purpose | Backing library |
| --- | --- | --- |
| `Button` | Primary, secondary, ghost, danger and compact buttons | native button |
| `IconButton` | Icon-only action with mandatory accessible label | native button + Lucide |
| `TextField` | Label, hint, error, input slot and generated id wiring | native input |
| `SecretInput` | Password-like input with reveal/generate slots | native input |
| `Dialog` | Modal content with focus trap and return focus | Bits UI |
| `AlertDialog` | Confirmation for destructive irreversible actions | Bits UI |
| `DropdownMenu` | Desktop menus and row overflow menus | Bits UI/Floating UI |
| `Popover` | Small anchored transient panels | Bits UI/Floating UI |
| `Tooltip` | Nonessential hover/focus help | Bits UI/Floating UI |
| `Sheet` | Mobile bottom sheet / drawer | Svelte 5 drawer primitive |
| `Tabs` | Detail sub-sections where all panels are peer-level | Bits UI |
| `Switch` | Boolean settings | Bits UI |
| `Checkbox` | Multi-select and settings checkboxes | Bits UI/native |
| `Toast` | Copy success, saved, undo and transport messages | custom + live region |

Primitive wrappers expose Castellan variant names and events. They should not
leak third-party library-specific prop names beyond the wrapper.

### §4.2 Product components to create in `packages/ui/src/components/`

| Component | Purpose |
| --- | --- |
| `LockStateIndicator` | Visible locked/unlocked/disconnected/no-database state |
| `SearchBox` | Search field with clear button, shortcut hint and loading state |
| `CommandPalette` | Keyboard-first command/search launcher |
| `VaultSidebar` | Desktop navigation over filters, groups, tags and trash |
| `EntryRow` | Metadata-only list row for one entry |
| `EntryList` | Virtualizable list region with empty/loading/error states |
| `CapabilityBadges` | TOTP, passkey, recovery, SSH and attachment markers |
| `EntryDetail` | Read-only field-local action view of one entry |
| `EntryEditor` | Sectioned create/edit form |
| `FieldRow` | Label/value/action row foundation |
| `SecretFieldRow` | Concealed secret display with reveal/copy |
| `TotpFieldRow` | TOTP code with countdown and copy |
| `UrlFieldRow` | URL display with open/copy |
| `PasskeyFieldRow` | RP ID, user handle label and hardware/soft marker |
| `RecoveryCodeGrid` | Concealed recovery code set with mark-used/undo |
| `PasswordGeneratorPanel` | Password/passphrase settings and output |
| `EntryActionsSheet` | Mobile quick actions for one entry |
| `TrashBanner` | Trash state with restore and permanent-delete affordances |
| `ConnectedDeviceRow` | Paired device/browser/session row |
| `ActivityLogView` | User-visible local action log with filters and export |
| `ActionLogEntryRow` | One redacted receipt row in activity/per-entry history |
| `ActionDetailPanel` | Full non-secret receipt detail and related links |
| `ActionReceiptToast` | Immediate toast that points to durable activity history |
| `SyncStatusCard` | This-device identity, discoverability and LAN-only state |
| `SyncDeviceCard` | Nearby/paired sync peer with change counts and trust state |
| `PairingRequestDialog` | Human-verifiable device pairing approval |
| `SyncReviewPanel` | Incoming/outgoing change summary and conflict choices |
| `SyncProgressPanel` | Per-stage and total sync progress with cancel/advanced log |
| `SyncHistoryList` | Local audit trail of sync attempts without secret values |
| `SyncSettingsPanel` | Receive policy, network interface and history settings |
| `AuditFindingCard` | Security finding summary and remediation action |
| `EmptyState` | Reusable empty surfaces with icon, copy and action |
| `ErrorState` | Recoverable error surface with retry/details |

### §4.3 Layout components to create in `packages/ui/src/layouts/`

| Component | Purpose |
| --- | --- |
| `DesktopVaultShell` | Three-pane desktop structure and responsive collapse points |
| `MobileVaultShell` | Header/search/content/bottom-nav frame |
| `SplitDetailShell` | Two-pane tablet/small desktop list-detail frame |
| `SettingsShell` | Settings sidebar/tabs plus detail content |
| `SyncShell` | Device identity, peers, requests, review and progress regions |

The browser extension popup shell stays in `apps/extension` unless a future task
adds a tiny extension-only shared package. Do not make `apps/extension` depend
on `@castellan/ui` just to reuse app chrome.

## §5 Component specifications

### §5.1 `Button`

Layout:

```text
[ Primary action ]
[ Secondary action ]
[ Delete permanently ]
```

Props/contract:

- variants: `primary`, `secondary`, `ghost`, `danger`, `link`;
- sizes: `compact`, `default`, `touch`;
- supports `disabled`, `loading`, `type` and accessible label/content.

Behavior:

- Native button semantics.
- Loading state disables repeated activation and announces progress if text
  changes.
- Danger variant is visual only; irreversible actions still require
  `AlertDialog` or a trash/undo step.

Tests:

- disabled cannot activate;
- loading has stable width or documented width behavior;
- focus ring visible.

### §5.2 `IconButton`

Layout:

```text
[👁] [Copy] [⋯]
```

Contract:

- Requires an accessible label.
- Tooltip is optional and must duplicate, not replace, the label.
- Icon-only controls use 44 px touch target on mobile.

Behavior:

- Enter/Space activates.
- Tooltip opens on hover/focus and closes on Escape.

### §5.3 `LockStateIndicator`

Layout:

```text
🔒 Locked
🔓 Unlocked
⚠ Disconnected
— No database
```

Contract:

- states: `locked`, `unlocked`, `disconnected`, `no_database`, `checking`;
- optional action slot for unlock, lock, reconnect or open database.

Behavior:

- State text must be visible, not color-only.
- Locked and disconnected states block secret-field actions.
- In compact chrome, icon + accessible label is acceptable if nearby text names
  the state elsewhere.

### §5.4 `SearchBox`

Layout:

```text
┌──────────────────────────────┐
│ 🔎 Search vault…         ⌘K │
└──────────────────────────────┘
```

Contract:

- props: `value`, `placeholder`, `shortcut`, `loading`, `disabled`;
- events/callbacks: value change, clear, submit.

Behavior:

- Clear button appears only when non-empty.
- `Escape` clears when non-empty; otherwise caller may close parent surface.
- Search never queries secrets that are excluded by policy, such as recovery
  code bodies.

### §5.5 `CommandPalette`

Layout:

```text
┌────────────────────────────────────┐
│ Search or run command…             │
├────────────────────────────────────┤
│ GitHub                         ↵   │
│ Generate password                  │
│ Lock vault                         │
└────────────────────────────────────┘
```

Contract:

- Input plus grouped results.
- Result kinds: entry, action, setting, device/browser, help.
- Each result has title, optional subtitle, optional icon and callback.

Behavior:

- Opens with `⌘K`/`Ctrl+K`.
- Arrow keys move active result; Enter activates; Escape closes.
- Focus returns to opener.
- Locked state shows unlock/open actions and hides secret operations.

### §5.6 `VaultSidebar`

Layout:

```text
Vault
  All entries
  Favorites
  Logins
  TOTP
  Passkeys
  Recovery codes
  SSH keys
  Audit
  Trash

Groups
  Personal
  Work

Tags
  banking
  shared
```

Contract:

- hierarchical nav groups with count badges where available;
- active item is represented with `aria-current` or selected state;
- collapsible sections preserve state per session.

Behavior:

- Keyboard arrow navigation within tree/list sections.
- Counts are metadata only and may be omitted while loading.
- Trash and audit are visually distinct from normal group filters.

### §5.7 `EntryRow`

Layout:

```text
┌────────────────────────────────────┐
│ GitHub                     2FA 🔑 │
│ ada@castellan.dev                 │
└────────────────────────────────────┘
```

Contract:

- props include metadata only: id, title, username/display subtitle, URL/host,
  favorite flag, type and capability booleans.
- optional selected, disabled, stale/error states.

Behavior:

- Click/tap selects or opens according to the owning surface.
- Enter/Space activates.
- Row overflow opens menu/sheet but does not trigger row activation.
- No secret values in props.

Tests:

- title and subtitle truncate rather than overflow;
- badges have accessible labels;
- keyboard activation calls `onPick` once.

### §5.8 `EntryList`

Layout:

```text
Entries
[ Search/filter chips ]
──────────────────────
GitHub            2FA 🔑
Email             2FA
Bank                  🔑
```

Contract:

- accepts rows, active id, loading, empty and error states;
- emits pick, range-select if multi-select is enabled, and request-more when
  virtualized/paginated.

Behavior:

- Maintains stable active descendant or roving focus.
- Empty state copy names the current filter, e.g. "No TOTP entries".
- Loading skeletons do not imply fake data.

### §5.9 `FieldRow`

Layout:

```text
Label
value text                                      [Action]
```

Contract:

- shared row for text-like fields;
- label, value slot, action slot, optional hint/error.

Behavior:

- Value wraps on mobile and may truncate on desktop when the owning detail view
  offers copy/open actions.
- Action buttons remain reachable when value wraps.

### §5.10 `SecretFieldRow`

Layout:

```text
Password
••••••••••••••••                         [Reveal] [Copy]
```

Contract:

- props: `revealed`, `disabled`, `copied`, optional strength label in edit
  contexts;
- callbacks: reveal/hide, copy.

Behavior:

- Concealed by default.
- Reveal does not copy.
- Copy does not reveal.
- Revealed state clears on lock, route change or explicit hide; the caller owns
  the timer and passes state down.
- Screen readers must not read a concealed secret as bullets forever; the row
  should expose label plus "concealed" unless explicitly revealed.

### §5.11 `TotpFieldRow`

Layout:

```text
One-time password
123 456                         21s ring      [Copy]
```

Contract:

- props: code, seconds remaining, period, loading/error/locked state;
- callbacks: copy, refresh if provided.

Behavior:

- Countdown is local between protocol answers.
- At zero, display stale/loading state until caller supplies a new code.
- Copy action is disabled when locked or stale.
- TOTP seed is never a prop.

### §5.12 `UrlFieldRow`

Layout:

```text
URL
https://github.com                         [Open] [Copy]
```

Behavior:

- Open action uses caller callback; component does not call `window.open`
  directly unless explicitly designed for a browser-only surface.
- Display may show host in compact mode and full URL in expanded mode.

### §5.13 `PasskeyFieldRow`

Layout:

```text
Passkey
github.com · soft key                       [Details]
```

Behavior:

- Shows RP ID and whether the credential is soft or hardware-bound.
- Never exposes private key material.
- For hardware-bound credentials, destructive/delete copy must warn that the
  credential may not be recoverable from sync alone.

### §5.14 `RecoveryCodeGrid`

Layout:

```text
Recovery codes
┌─────────────┬─────────────┐
│ ••••••••••  │ ••••••••••  │
│ unused Copy │ used Undo   │
└─────────────┴─────────────┘
```

Behavior:

- Codes concealed by default.
- Copy one code at a time.
- Mark-used has undo.
- Used state is visible and not color-only.
- Search indexes labels/issuer metadata, never code bodies.

### §5.15 `PasswordGeneratorPanel`

Layout:

```text
Generated password
correct-horse-battery-staple        [Copy]

Type          Passphrase ▾
Words         [4]
Separator     [-]
Options       ☑ numbers  ☑ symbols
Strength      Very strong · 156 bits
```

Behavior:

- Generate action calls through the face/client; this package does not generate
  secrets locally unless a future protocol decision moves that logic.
- Copy is explicit and does not auto-fill without user action.
- Settings are keyboard accessible and preserve last used values per face.

### §5.16 `EntryDetail`

Layout:

```text
┌────────────────────────────────────┐
│ GitHub                        Edit │
│ github.com                         │
├────────────────────────────────────┤
│ Username                      Copy │
│ ada@castellan.dev                  │
│ Password              Reveal Copy │
│ •••••••••••••••                   │
│ One-time password             Copy │
│ 123 456 · 21s                     │
│ URL                       Open Copy│
│ https://github.com                │
└────────────────────────────────────┘
```

Contract:

- read-only view composed from field rows;
- receives field data and action callbacks from caller;
- supports trash state, favorite state and stale/changed warning.

Behavior:

- Default focus goes to the heading or first primary action when opened from a
  list.
- Field actions remain available in both desktop and mobile layouts.
- Edit action is suppressed/disabled when vault is locked or read-only.

### §5.17 `EntryEditor`

Desktop layout:

```text
┌────────────────────────────────────────────────────────────────┐
│ Edit entry                                                Save │
├──────────────┬─────────────────────────────────────────────────┤
│ Entry        │ Title       [GitHub                         ]   │
│ Security     │ Username    [ada@castellan.dev              ]   │
│ TOTP         │ Password    [••••••••••••      🎲 👁        ]   │
│ Passkeys     │ Strength    Very strong                         │
│ Autofill     │ URL         [https://github.com              ]   │
│ History      │ Tags        [work ×] [security ×]               │
│ Advanced     │ Notes       [                               ]   │
└──────────────┴─────────────────────────────────────────────────┘
```

Mobile layout:

```text
┌──────────────────────────────┐
│ Cancel       Entry      Done │
├──────────────────────────────┤
│ Title                        │
│ [GitHub                    ] │
│ Username                     │
│ [ada@castellan.dev         ] │
│ Password                     │
│ [••••••••••••        🎲 👁 ] │
│ Very strong                  │
│ URL                          │
│ [https://github.com        ] │
│ Set up one-time password  ›  │
└──────────────────────────────┘
```

Behavior:

- Dirty state blocks accidental close with confirmation.
- Save errors stay near fields when field-specific and at top when global.
- Password generator is adjacent to password input.
- TOTP setup offers exactly: scan QR code, paste/enter URI manually, remove
  existing TOTP when present.
- Advanced sections do not hide required save/cancel controls.

### §5.18 `EntryActionsSheet`

Layout:

```text
┌──────────────────────────────┐
│ GitHub                       │
│ ada@castellan.dev            │
├──────────────────────────────┤
│ Fill                         │
│ Copy username                │
│ Copy password                │
│ Copy TOTP                    │
│ Edit                         │
│ Move to trash                │
└──────────────────────────────┘
```

Behavior:

- Mobile: bottom sheet. Desktop: dropdown/context menu.
- Copy password remains explicit and separated from copy username.
- Destructive actions are grouped at bottom and styled as danger.
- Sheet closes after successful single-shot action unless the action opens a
  child flow.

### §5.19 `TrashBanner`

Layout:

```text
┌──────────────────────────────────────────────┐
│ This item is in the trash. [Restore] [Delete]│
└──────────────────────────────────────────────┘
```

Behavior:

- Restore is primary/secondary; permanent delete is danger.
- Permanent delete opens `AlertDialog` with the item title.
- Banner appears in both detail and editor when viewing trash.

### §5.20 `ConnectedDeviceRow`

Layout:

```text
┌──────────────────────────────────────────────┐
│ Firefox profile · Fob                  Live │
│ Last request: 2 min ago · protocol v2       │
│ [Disconnect] [Forget]                       │
└──────────────────────────────────────────────┘
```

Behavior:

- Distinguishes browser extension sessions, paired sync devices and current
  device when all are eventually present.
- Pending association rows show human-verifiable fingerprint phrase before
  approval.
- Kill/forget actions are explicit and auditable.

### §5.21 `AuditFindingCard`

Layout:

```text
┌──────────────────────────────────────────────┐
│ Reused password                       High   │
│ 3 entries share this password.               │
│ [Review entries] [Generate replacement]      │
└──────────────────────────────────────────────┘
```

Behavior:

- Severity visible as text and color/icon.
- Never shows the compromised/reused password itself.
- Remediation actions open scoped lists or editor flows.

### §5.22 `EmptyState` and `ErrorState`

Layout:

```text
No entries for github.com
Save a login in the browser extension or create one here.
[Create entry]
```

Behavior:

- Empty copy names the filter/scope.
- Error copy exposes a short message and optional technical details disclosure.
- Retry action is present only when caller supplies a retry callback.


### §5.23 `SyncStatusCard`

Layout:

```text
┌──────────────────────────────────────────────┐
│ This device                                  │
│ Warsaw laptop                     Discoverable│
│ castle-river-copper-lantern                  │
│ LAN only · no cloud relay                    │
│ [Stop discovery] [Show QR]                   │
└──────────────────────────────────────────────┘
```

Behavior:

- Shows device alias, device type, discoverability/server state and
  human-verifiable fingerprint phrase.
- Never implies cloud availability; "LAN only" or equivalent copy is visible
  in the sync surface.
- Discoverability can be temporary. If a timeout exists, show remaining time.
- Network-blocked states suggest local-network permission, VPN/firewall and
  interface settings before suggesting a product bug.

### §5.24 `SyncDeviceCard`

Layout:

```text
┌──────────────────────────────────────────────┐
│ Pixel 9                              Paired  │
│ last sync 2m ago · protocol v2              │
│ 3 local changes · 1 remote change     [Sync] │
└──────────────────────────────────────────────┘
```

Behavior:

- Distinguishes paired, unpaired, pending, offline, incompatible-version and
  blocked devices.
- Primary action is `Sync` only for paired compatible devices.
- Unpaired devices offer `Pair`, which opens `PairingRequestDialog`; they do
  not receive vault data from a row click.
- Favorite/trusted-device markings may enable auto-accept, but the copy must
  say exactly what auto-accept can and cannot do.

### §5.25 `PairingRequestDialog`

Layout:

```text
┌──────────────────────────────┐
│ Pair Pixel tablet             │
│                              │
│ Confirm this phrase appears  │
│ on both devices:             │
│                              │
│ castle-river-copper-lantern  │
│                              │
│ [Decline]           [Approve]│
└──────────────────────────────┘
```

Behavior:

- Requires explicit approval on the trusted app side.
- Phrase/QR is text-visible and copy-resistant enough for shoulder checking;
  do not hide it in an icon-only interaction.
- Approval stores a pinned device fingerprint, display alias and last-seen
  metadata.
- Decline leaves no remembered trust. Repeated declined requests can be blocked.

### §5.26 `SyncReviewPanel`

Layout:

```text
Review sync from Pixel tablet
Added 2 · Modified 5 · Conflicts 1 · Deleted 0

Conflicts
GitHub
local password changed · remote TOTP changed
[Merge] [Keep local] [Use remote]

[Cancel] [Apply changes]
```

Behavior:

- Required before applying first sync from a device, any destructive change, any
  conflict and any schema/protocol downgrade.
- Change summaries show entry titles and metadata only; secret diffs are never
  displayed inline.
- Conflict rows provide choices per conflict and a safe default that preserves
  both sides when possible.
- Apply action names the count of changes it will write.

### §5.27 `SyncProgressPanel`

Layout:

```text
Syncing with Pixel tablet
✓ Verified device fingerprint
✓ Created copy-aside backup
▬ Receiving encrypted delta
· Merging entries
· Writing vault

Total progress
██████████░░░░░░░░░░
[Advanced log] [Cancel]
```

Behavior:

- Stages are semantic, not just bytes: discovery, handshake, fingerprint,
  backup, transfer, merge, write, verify and finish.
- Total progress is visible, but exact percentages may be omitted when the
  operation is not measurable.
- Cancel explains consequences based on current stage. Before write it cancels;
  during atomic write it waits; after write it cannot roll back without an
  explicit restore flow.
- Advanced log shows protocol/device details but no secret field values.

### §5.28 `SyncHistoryList`

Layout:

```text
Sync history
Today 14:32 · Pixel 9 · success · 7 changes
Today 09:10 · iPad · conflict unresolved
Yesterday · ThinkPad · failed: network timeout
```

Behavior:

- Records peer, time, direction, result and counts.
- Links to backup/restore review when a backup was created.
- Does not store or render secret values.
- Clear history is allowed only for local history and must not forget paired
  devices.

### §5.29 `SyncSettingsPanel`

Layout:

```text
Receive
Receive from paired devices      Ask each time ▾
Auto-apply non-conflicting syncs Off
Save sync history                On

Network
Discoverable on LAN              Off
Interface                        All trusted interfaces ▾
Port                             53317
```

Behavior:

- Receive policy choices are explicit: off, ask each time, auto-accept from
  paired devices. There is no auto-accept-from-everyone option for vault data.
- Advanced network settings are available but not required for the happy path.
- Interface allow/block lists are visible when discovery fails on machines with
  VPNs, multiple NICs or firewalls.
- Changing port/interface restarts the local server and reports success/failure.


### §5.30 `ActivityLogView`

Layout:

```text
┌──────────────────────────────────────────────────────────────┐
│ Activity                                         Export…      │
├──────────────────────────────────────────────────────────────┤
│ [All actions ▾] [Secret exposure] [This device ▾] [Search…] │
├──────────────────────────────────────────────────────────────┤
│ Today                                                        │
│ 14:32  Password copied       GitHub       desktop  success   │
│ 14:30  SSH key used          prod-admin   cli      success   │
│ 14:12  Sync applied          Pixel 9      sync     success   │
│ 13:55  Fill denied           evil.example ext      denied    │
└──────────────────────────────────────────────────────────────┘
```

Behavior:

- Shows user-visible receipts from the local action log; it is not a debug log.
- Default filters expose secret exposures, denied/failed actions, face, device,
  target kind and time range.
- Locked vault hides detailed vault action records; pre-unlock operational rows
  show only safe coarse state.
- Export opens a redaction review before writing a file. Exporting the log is
  itself logged.
- Search and filters operate on metadata only; secret values are never indexed
  because they are never in the records.

### §5.31 `ActionLogEntryRow`

Layout:

```text
14:32  Password copied       GitHub       desktop  success
```

Behavior:

- Row names action, target display, face/device and outcome in text, never color
  alone.
- `secret_exposed` rows have a clear marker such as "secret used" or "secret
  exposed" without revealing the value.
- Activating the row opens `ActionDetailPanel`.
- Rows must support dense desktop and readable mobile layouts.

### §5.32 `ActionDetailPanel`

Layout:

```text
Password copied
Today 14:32:08 +02:00

Entry: GitHub
Face: Desktop
Device: Warsaw laptop
Outcome: success
Secret value: not stored in log
Clipboard policy: clear after 30 seconds

[Show related entry] [Copy diagnostic summary]
```

Behavior:

- Always states that secret values are not stored when the action exposed a
  secret.
- Related-entry/device/project links are shown only when the current user can
  view that object.
- Diagnostic summary uses the diagnostics redaction policy by default.

### §5.33 `ActionReceiptToast`

Layout:

```text
Password copied · clears in 30s       [Activity]
```

Behavior:

- Immediate feedback for copy/fill/reveal/sync/apply actions.
- Links to the durable Activity view or relevant filtered history.
- Toast text and durable log vocabulary stay aligned: the action name a user
  sees in the toast should be findable in Activity.
- Toast disappearance does not delete the durable receipt.

## §6 Story and test matrix

Each reusable component should have stories/tests for the states below when
applicable:

| State | Story | Component test |
| --- | --- | --- |
| Default | yes | render/accessibility smoke |
| Hover/focus | visual story or pseudo-state | focus visible |
| Loading | yes | disabled/no double-submit |
| Empty | yes | copy/action visible |
| Error | yes | message and retry/details |
| Locked | yes for secret-bearing components | actions disabled |
| Mobile density | yes | 44 px touch target |
| Keyboard | optional story | Enter/Space/Escape/arrow behavior |
| Destructive | yes | confirmation or undo path |


## §7 Agent visual/spec conformance loop

A UI implementation task is not done when it compiles. The agent must be able to
see the rendered surface and compare it against this spec with the offline
browser loop. The goal is not pixel-perfect copying of reference apps; the goal
is to catch divergence from Castellan's own contracts before it becomes a new
implicit design.

### §7.1 Loop shape

```text
pick spec section
      ↓
render Storybook story or app surface
      ↓
open with offline Chromium
      ↓
inspect layout + interact with keyboard/pointer
      ↓
compare against this spec
      ↓
fix implementation or update spec with rationale
      ↓
rerun behavior tests and inspect again
```

### §7.2 Rendering target priority

1. **Storybook story** for component shape, states, density and isolated
   behavior.
2. **Playwright component harness** for keyboard contracts, events and focus.
3. **Desktop/mobile app shell** for responsive pane behavior, route transitions,
   face-owned metrics and safe-area choices.
4. **Extension popup or overlay harness** only for extension-specific flows;
   injected overlays should be tested inside Shadow DOM against hostile page CSS.

### §7.3 Offline browser procedure

From the repository root:

```console
$ bun run browsers:offline
$ bun run dev:storybook
$ bunx playwright-cli open --browser chromium http://127.0.0.1:6006
```

Use the local URL for the agent-driven offline browser. If the UI is being
shown to the user through Arena live preview, the server must bind `0.0.0.0`
and browser-facing requests must use relative URLs rather than `localhost`.

### §7.4 What to inspect

For each changed component or shell, check:

- **Layout:** matches the ASCII contract at the relevant breakpoint; no hidden
  primary action; field actions remain near their values.
- **State:** default, loading, empty, error, locked, disabled, trash and
  destructive states render distinctly without layout jumps.
- **Secrets:** list surfaces remain metadata-only; copy does not reveal; reveal
  is explicit and temporary; TOTP seed/passkey key material/recovery code bodies
  never leak into list/search surfaces.
- **Keyboard:** Tab order is logical; Enter/Space activate; Escape closes
  transient surfaces; arrows move active rows/options where specified.
- **Focus:** focus ring is visible; focus is trapped inside modal surfaces and
  returned to the opener after close.
- **Touch:** mobile targets are at least 44 px; bottom sheets are reachable and
  do not put destructive actions next to primary actions.
- **A11y text:** state is not color-only; icon-only controls have accessible
  labels; badges have names.
- **Responsive behavior:** desktop three-pane collapses intentionally; mobile
  stack and safe areas remain usable.

### §7.5 Divergence policy

When rendered UI diverges from this spec, classify it before changing code:

| Divergence | Action |
| --- | --- |
| Implementation bug | Fix component/layout and add or adjust a story/test. |
| Missing state in spec | Extend this document, then implement the state. |
| Better design discovered | Update this spec with rationale before broadening code. |
| Third-party primitive limitation | Wrap/patch it at the Castellan wrapper boundary or record a follow-up task. |
| Intentional temporary gap | Link the task that will close it and keep the current behavior visibly honest. |

Do not commit ad-hoc screenshots or downloaded reference images as proof. Keep
visual evidence only when a task explicitly introduces committed regression
baselines; otherwise delete temporary browser artifacts before finishing.

## §8 Implementation order

Recommended build order after TASK-47:

1. Primitives: `Button`, `IconButton`, `Dialog`, `AlertDialog`, `DropdownMenu`,
   `Sheet`, `Tooltip`, `TextField`, `Toast`.
2. Field components: `FieldRow`, `SecretFieldRow`, `TotpFieldRow`,
   `UrlFieldRow`, `CapabilityBadges`, `LockStateIndicator`.
3. List/detail: `EntryRow` revision, `EntryList`, `EntryDetail`,
   `EntryActionsSheet`, `TrashBanner`.
4. Shells: `DesktopVaultShell`, `MobileVaultShell`, `SettingsShell`.
5. Editors and advanced flows: `EntryEditor`, `PasswordGeneratorPanel`,
   `RecoveryCodeGrid`, `PasskeyFieldRow`, `ConnectedDeviceRow`,
   `AuditFindingCard`, `CommandPalette`.
6. Activity/action-log surfaces: `ActionReceiptToast`, `ActionLogEntryRow`,
   `ActionDetailPanel`, then `ActivityLogView`.
7. Sync/device mesh surfaces: `SyncStatusCard`, `SyncDeviceCard`,
   `PairingRequestDialog`, `SyncReviewPanel`, `SyncProgressPanel`,
   `SyncHistoryList`, `SyncSettingsPanel`, then `SyncShell`.

Do not migrate all existing UI at once. Land primitives and tests first, then
move one user-facing flow at a time so Storybook and Playwright failures point
to a small change.

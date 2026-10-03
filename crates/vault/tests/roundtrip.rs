//! The round-trip harness (task-8): open → save → reopen → compare every
//! parsed field, so "keepass-rs dropped a field" is a red build, not a
//! lost database (decision-3).
//!
//! The comparator walks both databases field by field — meta, groups,
//! entries, protected values, custom data, icons, autotype, history,
//! attachments, the recycle bin's deleted-objects — and reports every
//! difference it finds, named. A corpus case fails on ANY loss; the
//! proptest generates arbitrary databases and asserts the same invariant,
//! so a loss that no committed fixture happens to carry is still caught.
//!
//! The one field deliberately not compared is `Meta::generator`: it names
//! the program that wrote the file last, and *should* change when this
//! program saves. Everything else must survive.

use std::collections::BTreeMap;
use std::path::PathBuf;

use keepass::Database;
use keepass::DatabaseKey;
use keepass::db::{AutoType, Entry, EntryRef, GroupRef, History, Value};
use proptest::prelude::*;
use rstest::rstest;

/// Where the committed corpus lives.
fn corpus(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A unique scratch path per call.
fn scratch(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "castellan-roundtrip-{}-{tag}-{unique}.kdbx",
        std::process::id()
    ))
}

/// Parse a file with keepass directly — the harness's reader, deliberately
/// not the vault crate's handle: the comparator needs `&Database`, and the
/// save under test has already gone through the front door.
fn parse(
    path: &std::path::Path,
    password: Option<&str>,
    keyfile: Option<&std::path::Path>,
) -> Database {
    let mut source = std::fs::File::open(path).expect("open");
    let mut key = DatabaseKey::new();
    if let Some(password) = password {
        key = key.with_password(password);
    }
    if let Some(keyfile) = keyfile {
        let mut file = std::fs::File::open(keyfile).expect("open keyfile");
        key = key.with_keyfile(&mut file).expect("keyfile");
    }
    Database::open(&mut source, key).expect("parse")
}

// ── The comparator: every difference, named ──────────────────────────────────

/// Compare two parsed databases field by field. Returns the differences;
/// empty means the round trip lost nothing the parser can see.
fn compare_databases(original: &Database, saved: &Database) -> Vec<String> {
    let mut diffs = Vec::new();
    compare_meta(&original.meta, &saved.meta, &mut diffs);
    compare_group(original.root(), saved.root(), "/".into(), &mut diffs);
    if original.deleted_objects != saved.deleted_objects {
        diffs.push(format!(
            "deleted objects: {} -> {}",
            original.deleted_objects.len(),
            saved.deleted_objects.len()
        ));
    }
    if original.num_custom_icons() != saved.num_custom_icons() {
        diffs.push(format!(
            "custom icon pool: {} -> {}",
            original.num_custom_icons(),
            saved.num_custom_icons()
        ));
    }
    diffs
}

/// Meta minus `generator` — the one field a save is allowed to rewrite.
fn compare_meta(a: &keepass::db::Meta, b: &keepass::db::Meta, diffs: &mut Vec<String>) {
    macro_rules! field {
        ($name:ident) => {
            if a.$name != b.$name {
                diffs.push(format!(
                    "meta.{}: {:?} -> {:?}",
                    stringify!($name),
                    a.$name,
                    b.$name
                ));
            }
        };
    }
    field!(database_name);
    field!(database_name_changed);
    field!(database_description);
    field!(database_description_changed);
    field!(default_username);
    field!(default_username_changed);
    field!(maintenance_history_days);
    field!(color);
    field!(master_key_changed);
    field!(master_key_change_rec);
    field!(master_key_change_force);
    field!(memory_protection);
    field!(recyclebin_enabled);
    field!(recyclebin_uuid);
    field!(recyclebin_changed);
    field!(entry_templates_group);
    field!(entry_templates_group_changed);
    field!(last_selected_group);
    field!(last_top_visible_group);
    field!(history_max_items);
    field!(history_max_size);
    field!(settings_changed);
    compare_custom_data(&a.custom_data, &b.custom_data, "meta", diffs);
}

fn compare_group(a: GroupRef<'_>, b: GroupRef<'_>, path: String, diffs: &mut Vec<String>) {
    let here = format!("{path}{}/", a.name);
    if a.name != b.name {
        diffs.push(format!("{here}group name: {} -> {}", a.name, b.name));
    }
    if a.notes != b.notes {
        diffs.push(format!("{here}notes: {:?} -> {:?}", a.notes, b.notes));
    }
    if a.tags != b.tags {
        diffs.push(format!("{here}tags: {:?} -> {:?}", a.tags, b.tags));
    }
    if a.icon() != b.icon() {
        diffs.push(format!("{here}icon: {:?} -> {:?}", a.icon(), b.icon()));
    }
    if a.times != b.times {
        diffs.push(format!("{here}times: {:?} -> {:?}", a.times, b.times));
    }
    if a.custom_data != b.custom_data {
        diffs.push(format!("{here}custom data changed"));
        compare_custom_data(&a.custom_data, &b.custom_data, &here, diffs);
    }
    if a.is_expanded != b.is_expanded {
        diffs.push(format!(
            "{here}is_expanded: {} -> {}",
            a.is_expanded, b.is_expanded
        ));
    }
    if a.default_autotype_sequence != b.default_autotype_sequence {
        diffs.push(format!(
            "{here}default autotype: {:?} -> {:?}",
            a.default_autotype_sequence, b.default_autotype_sequence
        ));
    }
    if a.enable_autotype != b.enable_autotype {
        diffs.push(format!(
            "{here}enable_autotype: {:?} -> {:?}",
            a.enable_autotype, b.enable_autotype
        ));
    }
    if a.enable_searching != b.enable_searching {
        diffs.push(format!(
            "{here}enable_searching: {:?} -> {:?}",
            a.enable_searching, b.enable_searching
        ));
    }

    // Children compare positionally: KeePass order is user-visible state,
    // not an implementation detail.
    let a_entries: Vec<_> = a.entries().collect();
    let b_entries: Vec<_> = b.entries().collect();
    if a_entries.len() != b_entries.len() {
        diffs.push(format!(
            "{here}entry count: {} -> {}",
            a_entries.len(),
            b_entries.len()
        ));
    }
    for (index, (original, saved)) in a_entries.into_iter().zip(b_entries).enumerate() {
        compare_entry(original, saved, format!("{here}[{index}]"), diffs);
    }
    let a_groups: Vec<_> = a.groups().collect();
    let b_groups: Vec<_> = b.groups().collect();
    if a_groups.len() != b_groups.len() {
        diffs.push(format!(
            "{here}child group count: {} -> {}",
            a_groups.len(),
            b_groups.len()
        ));
    }
    for (original, saved) in a_groups.into_iter().zip(b_groups) {
        compare_group(original, saved, here.clone(), diffs);
    }
}

fn compare_entry(a: EntryRef<'_>, b: EntryRef<'_>, path: String, diffs: &mut Vec<String>) {
    let title = a.get_title().unwrap_or("(untitled)").to_string();
    let here = format!("{path}{title}");
    if a.id().uuid() != b.id().uuid() {
        diffs.push(format!("{here}: uuid changed"));
    }
    compare_fields(&a.fields, &b.fields, &here, diffs);
    compare_autotype(a.autotype.as_ref(), b.autotype.as_ref(), &here, diffs);
    if a.tags != b.tags {
        diffs.push(format!("{here}: tags {:?} -> {:?}", a.tags, b.tags));
    }
    if a.times != b.times {
        diffs.push(format!("{here}: times {:?} -> {:?}", a.times, b.times));
    }
    compare_custom_data(&a.custom_data, &b.custom_data, &here, diffs);
    if a.foreground_color != b.foreground_color {
        diffs.push(format!("{here}: foreground color changed"));
    }
    if a.background_color != b.background_color {
        diffs.push(format!("{here}: background color changed"));
    }
    if a.override_url != b.override_url {
        diffs.push(format!(
            "{here}: override url {:?} -> {:?}",
            a.override_url, b.override_url
        ));
    }
    if a.quality_check != b.quality_check {
        diffs.push(format!(
            "{here}: quality check {} -> {}",
            a.quality_check, b.quality_check
        ));
    }
    if a.icon() != b.icon() {
        diffs.push(format!("{here}: icon {:?} -> {:?}", a.icon(), b.icon()));
    }
    // Attachments by name, with their bytes: the binary pool is the part
    // of a vault no projection ever shows and no user forgives losing.
    let a_attachments: BTreeMap<String, Vec<u8>> = a
        .attachments_named()
        .map(|(name, attachment)| (name.to_string(), attachment.data.get().clone()))
        .collect();
    let b_attachments: BTreeMap<String, Vec<u8>> = b
        .attachments_named()
        .map(|(name, attachment)| (name.to_string(), attachment.data.get().clone()))
        .collect();
    if a_attachments != b_attachments {
        diffs.push(format!(
            "{here}: attachments {a_attachments:?} -> {b_attachments:?}"
        ));
    }
    compare_history(a.history.as_ref(), b.history.as_ref(), &here, diffs);
}

/// History entries are plain [`Entry`] values (no database behind them),
/// so their comparison covers the fields that live on the entry itself.
fn compare_history(a: Option<&History>, b: Option<&History>, here: &str, diffs: &mut Vec<String>) {
    let (a_entries, b_entries) = match (a, b) {
        (Some(a), Some(b)) => (a.get_entries(), b.get_entries()),
        (None, None) => return,
        (a, b) => {
            diffs.push(format!("{here}: history presence {a:?} -> {b:?}"));
            return;
        }
    };
    if a_entries.len() != b_entries.len() {
        diffs.push(format!(
            "{here}: history depth {} -> {}",
            a_entries.len(),
            b_entries.len()
        ));
    }
    for (index, (original, saved)) in a_entries.iter().zip(b_entries.iter()).enumerate() {
        let entry_path = format!("{here} (history[{index}])");
        if original.id().uuid() != saved.id().uuid() {
            diffs.push(format!("{entry_path}: uuid changed"));
        }
        compare_fields(&original.fields, &saved.fields, &entry_path, diffs);
        compare_autotype(
            original.autotype.as_ref(),
            saved.autotype.as_ref(),
            &entry_path,
            diffs,
        );
        if original.tags != saved.tags {
            diffs.push(format!("{entry_path}: tags changed"));
        }
        if original.times != saved.times {
            diffs.push(format!("{entry_path}: times changed"));
        }
        compare_custom_data(
            &original.custom_data,
            &saved.custom_data,
            &entry_path,
            diffs,
        );
        if original.quality_check != saved.quality_check {
            diffs.push(format!("{entry_path}: quality check changed"));
        }
        if original.icon() != saved.icon() {
            diffs.push(format!("{entry_path}: icon changed"));
        }
    }
}

/// Every field key and every value, protected or not. `Value`'s equality
/// distinguishes protected from unprotected, so a field that loses its
/// protection flag is a difference here too.
fn compare_fields(
    a: &std::collections::HashMap<String, Value<String>>,
    b: &std::collections::HashMap<String, Value<String>>,
    here: &str,
    diffs: &mut Vec<String>,
) {
    for key in a.keys().filter(|key| !b.contains_key(*key)) {
        diffs.push(format!("{here}: field {key:?} lost"));
    }
    for key in b.keys().filter(|key| !a.contains_key(*key)) {
        diffs.push(format!("{here}: field {key:?} appeared"));
    }
    for (key, original) in a {
        if let Some(saved) = b.get(key) {
            if original != saved {
                diffs.push(format!("{here}: field {key:?} value changed"));
            }
        }
    }
}

fn compare_custom_data(
    a: &std::collections::HashMap<String, keepass::db::CustomDataItem>,
    b: &std::collections::HashMap<String, keepass::db::CustomDataItem>,
    here: &str,
    diffs: &mut Vec<String>,
) {
    for key in a.keys().filter(|key| !b.contains_key(*key)) {
        diffs.push(format!("{here}: custom data {key:?} lost"));
    }
    for (key, original) in a {
        if let Some(saved) = b.get(key) {
            if original != saved {
                diffs.push(format!("{here}: custom data {key:?} changed"));
            }
        }
    }
}

fn compare_autotype(
    a: Option<&AutoType>,
    b: Option<&AutoType>,
    here: &str,
    diffs: &mut Vec<String>,
) {
    let (Some(a), Some(b)) = (a, b) else {
        if a.is_some() != b.is_some() {
            diffs.push(format!("{here}: autotype presence changed"));
        }
        return;
    };
    if a.enabled != b.enabled
        || a.default_sequence != b.default_sequence
        || a.data_transfer_obfuscation != b.data_transfer_obfuscation
        || a.associations != b.associations
    {
        diffs.push(format!("{here}: autotype settings changed"));
    }
}

// ── The corpus: real files, every field, on every save ───────────────────────

/// One corpus file and the key that opens it.
struct CorpusFile {
    file: &'static str,
    password: Option<&'static str>,
    keyfile: Option<&'static str>,
}

/// The KDBX 4 corpus: every cipher/KDF combination, keyfile shapes, TOTP,
/// recycle-bin state, and the KeePassXC-2.7.12-authored 4.1 file. The KDBX
/// 3 fixture has its own test in `save.rs` (read-compat, refuse to save).
const CORPUS: &[CorpusFile] = &[
    CorpusFile {
        file: "test_db_kdbx4_with_password_argon2.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_argon2id.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_argon2id_chacha20.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_argon2id_twofish.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_argon2_chacha20.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_argon2_twofish.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_aes.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_password_deleted_entry.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    // Authored by KeePassXC 2.7.12 — the anchor fixture.
    CorpusFile {
        file: "test_db_kdbx41_features.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx41_with_password_aes.kdbx",
        password: Some("demopass"),
        keyfile: None,
    },
    CorpusFile {
        file: "test_db_kdbx4_with_keyfile.kdbx",
        password: None,
        keyfile: Some("test_key.key"),
    },
    CorpusFile {
        file: "test_db_kdbx4_with_keyfile_v2.kdbx",
        password: Some("demopass"),
        keyfile: Some("test_db_kdbx4_with_keyfile_v2.keyx"),
    },
];

#[rstest]
#[case(&CORPUS[0])]
#[case(&CORPUS[1])]
#[case(&CORPUS[2])]
#[case(&CORPUS[3])]
#[case(&CORPUS[4])]
#[case(&CORPUS[5])]
#[case(&CORPUS[6])]
#[case(&CORPUS[7])]
#[case(&CORPUS[8])]
#[case(&CORPUS[9])]
#[case(&CORPUS[10])]
#[case(&CORPUS[11])]
fn every_corpus_file_round_trips_without_field_loss(#[case] case: &CorpusFile) {
    let work = scratch("corpus");
    std::fs::copy(corpus(case.file), &work).expect("copy fixture to scratch");
    let keyfile = case.keyfile.map(corpus);
    let pre_save_bytes = std::fs::read(&work).expect("read fixture copy");

    // The front door, both ways: open, save, reopen.
    let mut handle =
        castellan_vault::open(&work, case.password, keyfile.as_deref()).expect("fixture opens");
    let outcome = handle.save().expect("save must succeed");
    castellan_vault::open(&work, case.password, keyfile.as_deref())
        .expect("the saved file reopens through the front door");

    // The copy-aside preserved the pre-save bytes.
    let aside = outcome.aside.expect("a prior file means a copy-aside");
    assert_eq!(
        std::fs::read(&aside).expect("aside readable"),
        pre_save_bytes
    );

    // Field by field: nothing the parser can see was lost.
    let original = parse(&corpus(case.file), case.password, keyfile.as_deref());
    let saved = parse(&work, case.password, keyfile.as_deref());
    let diffs = compare_databases(&original, &saved);
    assert!(
        diffs.is_empty(),
        "{}: the round trip lost or changed fields:\n{}",
        case.file,
        diffs.join("\n")
    );
}

/// The boundary the property excludes, pinned so it cannot move silently:
/// keepass-rs's `cs_opt_string` deserializer folds a whitespace-only
/// unprotected `<Value>` (here: `" "`) to `None`, and `xml_to_db_handle`
/// then materializes `""`. The bytes on disk are written verbatim, so
/// KeePassXC and Strongbox see the original value — this is a
/// parser-layer normalization on our side, not file corruption. If this
/// test ever fails, keepass-rs changed the fold; update the strategy
/// boundary in `unprotected_text` to match.
#[test]
fn whitespace_only_unprotected_values_fold_through_the_parser() {
    // A seed vault authored with keepass directly (the only writer of the
    // shape under test), then round-tripped through the front door.
    let work = scratch("fold");
    let mut seed = Database::new();
    {
        let mut root = seed.root_mut();
        let mut entry = root.add_entry();
        entry.set_unprotected("Title", "kept: content with edges");
        entry.set_unprotected("UserName", " ");
        entry.set_protected("Password", " \t\r\n still exact ");
    }
    let mut file = std::fs::File::create(&work).expect("create seed");
    seed.save(&mut file, DatabaseKey::new().with_password("pw"))
        .expect("write seed");

    let mut handle = castellan_vault::open(&work, Some("pw"), None).expect("seed opens");
    handle.save().expect("save must succeed");
    let reopened = parse(&work, Some("pw"), None);

    let root = reopened.root();
    let entry = root.entries().next().expect("the entry survived");
    assert_eq!(entry.get("UserName"), Some("")); // folded: " " -> ""
    // Edge spaces around content survive.
    assert_eq!(entry.get("Title"), Some("kept: content with edges"));
    // Protected: byte-exact, even \r and control characters.
    assert_eq!(entry.get("Password"), Some(" \t\r\n still exact "));
}

// ── The property: arbitrary databases lose nothing either ────────────────────

// The field strategies (`text`, `nonempty_text`, `tag`, `xml_char`) are
// defined with the generator below; text drawing is constrained to the
// charset the KDBX XML layer can hold without normalizing it away.

/// A tag as KeePass clients store them: trimmed, non-empty, no delimiter
/// characters — the `<Tags>` element is semicolon-separated and every
/// client trims on entry, so whitespace-only or delimited tags are shapes
/// the format itself cannot hold.
fn tag() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<u8>(), 1..10).prop_map(|bytes| {
        bytes
            .into_iter()
            .map(|b| char::from(b'a' + (b % 24)))
            .collect()
    })
}

/// An entry-shaped strategy: the fields a KeePass client actually writes,
/// plus a custom field and a tag or two so the projection has something to
/// trip over.
/// Title and UserName are unprotected; Password and the custom field are
/// protected — each side gets its own charset reality.
fn entry_data() -> impl Strategy<Value = (String, String, String, Vec<String>, String)> {
    (
        unprotected_text(),
        unprotected_text(),
        protected_text(),
        proptest::collection::vec(tag(), 0..3),
        protected_text(),
    )
}

/// Non-empty text for fields where an empty value is out of domain: the
/// writer normalizes an empty database name to "absent" (`Some("")` on
/// disk is a shape no KeePass client produces), and the harness is for
/// catching loss, not representation trivia.
fn nonempty_text() -> impl Strategy<Value = String> {
    proptest::collection::vec(xml_char(), 1..24).prop_map(|chars| chars.into_iter().collect())
}

/// A single character the KDBX XML layer can hold without normalizing it
/// away: XML 1.0 forbids most control characters, and its readers turn
/// `\r` into `\n` — rules every KeePass client inherits, so the strategy
/// stays inside them.
fn xml_char() -> impl Strategy<Value = char> {
    any::<char>().prop_filter("XML-stable character", |&c| {
        c == '\t' || c == '\n' || ('\u{20}'..='\u{FFFD}').contains(&c)
    })
}

/// Arbitrary field text, drawn from the XML-stable charset.
fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(xml_char(), 0..24).prop_map(|chars| chars.into_iter().collect())
}

/// Unprotected field text: any XML-stable string that is not entirely
/// whitespace. keepass-rs's `cs_opt_string` reader folds a whitespace-only
/// `<Value>` to `None` (the file bytes are verbatim — KeePassXC preserves
/// them; only our parser-layer view folds), so that shape is pinned by a
/// characterization test below instead of the property. The empty string
/// stays in domain: it round-trips exactly.
fn unprotected_text() -> impl Strategy<Value = String> {
    text().prop_filter("not all-whitespace", |s| s.trim() != "" || s.is_empty())
}

/// Protected field text: the full charset, no XML constraints — protected
/// values are inner-encrypted and base64-wrapped, so control characters
/// and `\r` survive the XML layer untouched.
fn protected_text() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..24).prop_map(|chars| chars.into_iter().collect())
}

/// A generated database: a root with a few groups, each with a few
/// entries, protected and unprotected fields, tags, autotype, and a
/// history snapshot on some entries — the shapes the corpus cannot
/// cover combinatorially.
fn generated_database() -> impl Strategy<Value = Database> {
    (
        nonempty_text(),
        proptest::collection::vec(
            (text(), proptest::collection::vec(entry_data(), 0..4)),
            0..3,
        ),
    )
        .prop_map(|(name, groups)| {
            let mut database = Database::new();
            database.meta.database_name = Some(name);
            let mut root = database.root_mut();
            for (group_name, entries) in groups {
                let mut group = root.add_group();
                group.name = group_name;
                group.notes = Some("group notes".into());
                group.times.expires = Some(false);
                for (title, username, password, tags, custom) in entries {
                    // A history snapshot, the way KeePass itself makes one:
                    // the entry's own earlier state, cloned before the edit,
                    // carrying the entry's own uuid (a history entry whose
                    // uuid matches a *different* live entry is a shape no
                    // client writes and keepass-rs's reader cannot resolve).
                    let mut entry = group.add_entry();
                    entry.set_unprotected("Title", "an older version");
                    entry.set_protected("Password", "an older secret");
                    let snapshot = Entry::clone(&entry);
                    entry.set_unprotected("Title", title);
                    entry.set_unprotected("UserName", username);
                    entry.set_protected("Password", password);
                    entry.set_unprotected("Notes", "line one\nline two");
                    entry.set_protected("custom.field", custom);
                    entry.tags = tags;
                    entry.quality_check = true;
                    entry.autotype = Some(AutoType {
                        enabled: true,
                        default_sequence: Some("{USERNAME}{TAB}{PASSWORD}".into()),
                        data_transfer_obfuscation: Default::default(),
                        associations: vec![keepass::db::AutoTypeAssociation {
                            window: "*Editor".into(),
                            sequence: "{PASSWORD}".into(),
                        }],
                    });
                    entry
                        .history
                        .get_or_insert_with(History::default)
                        .add_entry(snapshot);
                }
            }
            database
        })
}

proptest! {
    // Each case pays four Argon2 derivations (seed save, open, save,
    // reopen); 50 cases is the pinned budget for that cost.
    #![proptest_config(ProptestConfig::with_cases(50))]
    /// For any database the model can build: save through the front door,
    /// reopen, and lose nothing — the same invariant the corpus carries,
    /// over the shapes no fixture happens to contain. The copy-aside
    /// holds the pre-save bytes on every one of those saves, too.
    #[test]
    fn generated_databases_round_trip_without_field_loss(
        database in generated_database(),
        password in text(),
    ) {
        let work = scratch("proptest");
        let password = if password.is_empty() { "x".to_string() } else { password };
        {
            let mut file = std::fs::File::create(&work).expect("create seed");
            database.save(&mut file, DatabaseKey::new().with_password(&password)).expect("seed save");
        }
        let pre_save_bytes = std::fs::read(&work).expect("read seed");

        let mut handle = castellan_vault::open(&work, Some(&password), None).expect("seed opens");
        let outcome = handle.save().expect("save must succeed");
        let aside = outcome.aside.expect("prior file, so an aside");
        prop_assert_eq!(std::fs::read(&aside).expect("aside readable"), pre_save_bytes);

        let saved = parse(&work, Some(password.as_str()), None);
        let diffs = compare_databases(&database, &saved);
        prop_assert!(diffs.is_empty(), "field loss: {:?}", diffs);
    }
}

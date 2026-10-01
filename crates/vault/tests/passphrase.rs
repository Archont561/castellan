//! Passphrase generation: shape, membership, and the tripwire against a
//! frozen RNG — all through the public `passphrase` function and the
//! public wordlist it draws from.

use castellan_vault::{WORDS, passphrase};
use proptest::prelude::*;
use rstest::rstest;

#[rstest]
#[case(1, "-")]
#[case(5, "-")]
#[case(12, ".")]
#[case(20, "_")]
#[case(3, " ")]
fn passphrase_has_the_requested_shape(#[case] words: usize, #[case] separator: &str) {
    let phrase = passphrase(words, separator);
    let split: Vec<&str> = phrase.split(separator).collect();
    assert_eq!(split.len(), words);
    assert!(split.iter().all(|word| WORDS.contains(word)));
}

#[test]
fn different_calls_produce_different_phrases() {
    // Not a proof of randomness; a tripwire for a accidentally
    // deterministic seed (a frozen counter, a cached buffer).
    let a = passphrase(6, ".");
    let b = passphrase(6, ".");
    assert_ne!(a, b);
}

proptest! {
    /// For any word count and any separator the protocol can request,
    /// the phrase is exactly that many words from the list, joined by
    /// that separator — the contract `generate_passphrase` promises.
    #[test]
    fn passphrase_words_come_from_the_list(
        words in 1usize..=32,
        separator in prop_oneof![
            Just("-"),
            Just("."),
            Just("_"),
            Just(" "),
        ],
    ) {
        let phrase = passphrase(words, separator);
        let split: Vec<&str> = phrase.split(separator).collect();
        prop_assert_eq!(split.len(), words);
        prop_assert!(split.iter().all(|word| WORDS.contains(word)));
    }
}

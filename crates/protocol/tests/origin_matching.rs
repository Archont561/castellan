//! Origin matching: the one rule every face applies to "does this entry
//! fill here?". Integration tests over the public `matching` module —
//! the same function the app answers with and the WASM build pre-filters
//! with, so a disagreement here is a disagreement everywhere.

use castellan_protocol::matching::origin_matches;
use proptest::prelude::*;
use rstest::rstest;

#[rstest]
#[case("github.com", "https://github.com/login", true)]
#[case("https://github.com", "github.com", true)]
#[case("GITHUB.COM", "github.com", true)]
#[case("login.example.org", "example.org", true)]
#[case("example.org", "login.example.org", false)]
#[case("evil.example.org", "login.example.org", false)]
// Two labels deep is not "a" subdomain under the single-label rule —
// a.b.example.org is more likely a distinct service than example.org.
#[case("a.b.example.org", "example.org", false)]
#[case("localhost:3000", "localhost:8080", false)]
#[case("localhost:3000", "localhost", true)]
#[case("https://github.com/login", "https://github.com", true)]
fn origin_matching_matrix(#[case] origin: &str, #[case] candidate: &str, #[case] expected: bool) {
    assert_eq!(origin_matches(origin, candidate), expected);
}

/// A lowercase alphabetic label: `a`–`z`, 1–11 chars. Hosts built from
/// these avoid every edge `split_host_port` deliberately passes through
/// (ports, userinfo, IPv6), so the properties below test the matching
/// rule, not the parser.
fn label() -> impl Strategy<Value = String> {
    (any::<u8>(), prop::collection::vec(any::<u8>(), 0..10)).prop_map(|(first, rest)| {
        std::iter::once(first)
            .chain(rest)
            .map(|b| char::from(b'a' + (b % 26)))
            .collect()
    })
}

/// A 2–4 label host: `example.org` shape.
fn host() -> impl Strategy<Value = String> {
    (label(), label(), prop::collection::vec(label(), 0..2)).prop_map(|(first, second, more)| {
        std::iter::once(first)
            .chain(std::iter::once(second))
            .chain(more)
            .collect::<Vec<_>>()
            .join(".")
    })
}

proptest! {
    /// The reflexive case, in both directions the callers write it:
    /// bare host, and URL-shaped.
    #[test]
    fn a_host_matches_itself(host in host()) {
        let url_shaped = format!("https://{host}/login?next=/");
        prop_assert!(origin_matches(&host, &host));
        prop_assert!(origin_matches(&url_shaped, &host));
        prop_assert!(origin_matches(&host, &url_shaped));
    }

    /// The asymmetry the rule is named for: a single-label child page
    /// matches the parent entry, never the other way around.
    #[test]
    fn child_matches_parent_but_not_vice_versa(
        parent in host(),
        child_label in label(),
    ) {
        let child = format!("{child_label}.{parent}");
        prop_assert!(origin_matches(&child, &parent));
        prop_assert!(!origin_matches(&parent, &child));
    }

    /// Siblings under a shared parent never match each other.
    #[test]
    fn siblings_never_match(
        parent in host(),
        a in label(),
        b in label(),
    ) {
        let b = if a == b { format!("{a}x") } else { b };
        let a_host = format!("{a}.{parent}");
        let b_host = format!("{b}.{parent}");
        prop_assert!(!origin_matches(&a_host, &b_host));
    }
}

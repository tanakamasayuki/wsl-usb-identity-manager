//! Rules for handing a device to WSL without being asked.
//!
//! A rule names *what* to attach, and the four kinds differ in how firmly they
//! name it. That difference is the whole content of this module: matching is a
//! string comparison, but which string you compare decides whether the rule
//! still means what it meant yesterday.
//!
//! Nothing here reaches a device. A rule is evaluated against values the
//! enumeration already produced, so requirement R9.4 — no probing to decide an
//! automatic attach — holds by construction.

use serde::{Deserialize, Serialize};

/// What a rule matches on.
///
/// Declared most specific first. [`matching`] walks a device's candidates in
/// this order, so a device covered by both its identity and its VID/PID is
/// reported as matched by its identity — the more specific answer is the more
/// useful one to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// What the board itself answered, e.g. `esp32-s3-3485188f6d7c`.
    ///
    /// The only kind that names the board rather than something in front of it.
    /// It is also only known while the device is identified (R4.3), so a rule
    /// on it fires only after a probe has run this session — never before, and
    /// never by causing one.
    Identity,
    /// The USB serial number.
    ///
    /// Stable, and stable about the wrong thing when there is an adapter: it
    /// names the CH343 or the WCH-Link, not the board wired to it.
    Serial,
    /// Every device of a kind: "attach any CH340".
    VidPid,
    /// Whatever is at this bus id.
    ///
    /// **The bus id is not a stable name** (finding F1): it is a hub number
    /// Windows assigns in enumeration order plus a port number, so plugging in
    /// a dock renumbers it and the same bus id comes to mean a different
    /// device. Useful for "whatever I plug into this port", and honest about
    /// nothing more than that.
    BusId,
}

impl RuleKind {
    /// Every kind, most specific first.
    pub const ALL: [RuleKind; 4] = [Self::Identity, Self::Serial, Self::VidPid, Self::BusId];
}

/// One rule: attach whatever answers to this value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub kind: RuleKind,
    pub value: String,
}

impl Rule {
    pub fn new(kind: RuleKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into().trim().to_owned(),
        }
    }

    /// Whether this rule names `value`.
    ///
    /// Case-insensitive, because these values are typed by hand in the rules
    /// panel as well as picked off a device, and `1A86:7523` is not a different
    /// piece of hardware from `1a86:7523`. The stored text keeps whatever case
    /// it was given, so what the panel shows is what the user wrote.
    pub fn matches(&self, value: &str) -> bool {
        self.value.eq_ignore_ascii_case(value.trim())
    }

    fn is_same(&self, other: &Rule) -> bool {
        self.kind == other.kind && self.matches(&other.value)
    }
}

/// One thing about a device that a rule could name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub kind: RuleKind,
    pub value: String,
}

impl Candidate {
    pub fn new(kind: RuleKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }
}

/// Which kind of rule covers a device, if any.
///
/// `candidates` is expected in [`RuleKind::ALL`] order — most specific first —
/// which is how [`crate::snapshot`]'s consumers build it.
pub fn matching(rules: &[Rule], candidates: &[Candidate]) -> Option<RuleKind> {
    candidates
        .iter()
        .find(|candidate| {
            rules
                .iter()
                .any(|rule| rule.kind == candidate.kind && rule.matches(&candidate.value))
        })
        .map(|candidate| candidate.kind)
}

/// Drops blanks and duplicates, keeping the first of each.
///
/// Applied to whatever arrives from the frontend, so the stored file cannot
/// collect two rules that say the same thing in different case, and an empty
/// value — which would match nothing and read as a mistake — never reaches it.
pub fn sanitise(rules: Vec<Rule>) -> Vec<Rule> {
    let mut kept: Vec<Rule> = Vec::with_capacity(rules.len());
    for rule in rules {
        let rule = Rule::new(rule.kind, rule.value);
        if rule.value.is_empty() || kept.iter().any(|k| k.is_same(&rule)) {
            continue;
        }
        kept.push(rule);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<Candidate> {
        vec![
            Candidate::new(RuleKind::Identity, "esp32-s3-3485188f6d7c"),
            Candidate::new(RuleKind::Serial, "5B5F090816"),
            Candidate::new(RuleKind::VidPid, "1a86:7523"),
            Candidate::new(RuleKind::BusId, "12-3"),
        ]
    }

    #[test]
    fn a_rule_matches_its_own_kind_only() {
        // The same text under a different kind is a different rule: a serial
        // number that happens to read like a bus id must not attach a port.
        let rules = vec![Rule::new(RuleKind::Serial, "12-3")];
        assert_eq!(matching(&rules, &candidates()), None);
    }

    #[test]
    fn the_most_specific_match_is_the_one_reported() {
        let rules = vec![
            Rule::new(RuleKind::VidPid, "1a86:7523"),
            Rule::new(RuleKind::Identity, "esp32-s3-3485188f6d7c"),
        ];
        // Declaration order in the rule list must not decide this; the
        // candidate order does.
        assert_eq!(matching(&rules, &candidates()), Some(RuleKind::Identity));
    }

    #[test]
    fn case_does_not_make_a_different_rule() {
        let rules = vec![Rule::new(RuleKind::VidPid, "1A86:7523")];
        assert_eq!(matching(&rules, &candidates()), Some(RuleKind::VidPid));
    }

    #[test]
    fn a_device_without_a_candidate_cannot_be_matched_on_it() {
        // No probe has run, so there is no identity to match. A rule on one
        // waits rather than firing on something else.
        let without_identity = &candidates()[1..];
        let rules = vec![Rule::new(RuleKind::Identity, "esp32-s3-3485188f6d7c")];
        assert_eq!(matching(&rules, without_identity), None);
    }

    #[test]
    fn sanitising_drops_blanks_and_repeats() {
        let rules = vec![
            Rule::new(RuleKind::VidPid, "1a86:7523"),
            Rule::new(RuleKind::VidPid, "  1A86:7523 "),
            Rule::new(RuleKind::Serial, "   "),
            Rule::new(RuleKind::BusId, "12-3"),
        ];
        let kept = sanitise(rules);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].value, "1a86:7523", "the first spelling is kept");
        assert_eq!(kept[1].kind, RuleKind::BusId);
    }

    #[test]
    fn nothing_matches_an_empty_rule_list() {
        assert_eq!(matching(&[], &candidates()), None);
    }
}

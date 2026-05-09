use persona_mind::{ClaimScope, ClaimState, PersonaRole};

#[test]
fn claim_state_records_scope_once() {
    let mut state = ClaimState::new(PersonaRole::operator());
    let scope = ClaimScope::new("/tmp/persona");

    state.claim(scope.clone());
    state.claim(scope.clone());

    assert!(state.owns(&scope));
    assert_eq!(state.role().as_str(), "operator");
    assert_eq!(state.scope_count(), 1);
}

#[test]
fn claim_scope_normalizes_duplicate_separators_and_dot_segments() {
    let scope = ClaimScope::new("/git//github.com/./LiGoldragon/persona/");

    assert_eq!(scope.as_str(), "/git/github.com/LiGoldragon/persona");
}

#[test]
fn parent_claim_owns_child_paths_without_claiming_siblings() {
    let mut state = ClaimState::new(PersonaRole::operator());
    state.claim(ClaimScope::new("/git/github.com/LiGoldragon/persona"));

    assert!(state.owns(&ClaimScope::new(
        "/git/github.com/LiGoldragon/persona/src/lib.rs"
    )));
    assert!(!state.owns(&ClaimScope::new(
        "/git/github.com/LiGoldragon/persona-router/src/lib.rs"
    )));
}

#[test]
fn parent_claim_collapses_redundant_child_claims() {
    let mut state = ClaimState::new(PersonaRole::operator());
    state.claim(ClaimScope::new("/git/github.com/LiGoldragon/persona/src"));
    state.claim(ClaimScope::new("/git/github.com/LiGoldragon/persona"));

    assert!(state.owns(&ClaimScope::new(
        "/git/github.com/LiGoldragon/persona/src/lib.rs"
    )));
    assert_eq!(state.scope_count(), 1);
}

#[test]
fn overlapping_scopes_are_detected_symmetrically() {
    let parent = ClaimScope::new("/git/github.com/LiGoldragon/persona");
    let child = ClaimScope::new("/git/github.com/LiGoldragon/persona/src/lib.rs");
    let sibling = ClaimScope::new("/git/github.com/LiGoldragon/persona-router/src/lib.rs");

    assert!(parent.overlaps(&child));
    assert!(child.overlaps(&parent));
    assert!(!parent.overlaps(&sibling));
}

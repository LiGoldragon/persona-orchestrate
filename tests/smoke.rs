use persona_orchestrate::{ClaimScope, ClaimState, PersonaRole};

#[test]
fn claim_state_records_scope_once() {
    let mut state = ClaimState::new(PersonaRole::operator());
    let scope = ClaimScope::new("/tmp/persona");

    state.claim(scope.clone());
    state.claim(scope.clone());

    assert!(state.owns(&scope));
    assert_eq!(state.role().as_str(), "operator");
}

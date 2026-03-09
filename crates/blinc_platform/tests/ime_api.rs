use blinc_platform::{
    current_ime_state, set_ime_state, ImeCompositionSelection, ImeCursorArea, ImeRequest, ImeState,
    ImeVisibility, SelectionRange, TextInputSessionId,
};

#[test]
fn ime_request_captures_mobile_runtime_contract_fields() {
    let request = ImeRequest::new(TextInputSessionId::new("search"))
        .with_visibility(ImeVisibility::Visible)
        .with_cursor_area(ImeCursorArea::new(10.0, 20.0, 30.0, 40.0))
        .with_selection(SelectionRange::new(2, 5))
        .with_composition(Some(ImeCompositionSelection::new(0, 1)));

    assert_eq!(request.session, TextInputSessionId::new("search"));
    assert_eq!(request.visibility, ImeVisibility::Visible);
    assert_eq!(request.selection, Some(SelectionRange::new(2, 5)));
    assert_eq!(
        request.composition,
        Some(ImeCompositionSelection::new(0, 1))
    );
    assert_eq!(
        request.cursor_area,
        Some(ImeCursorArea::new(10.0, 20.0, 30.0, 40.0))
    );
}

#[test]
fn ime_state_round_trips_request_metadata() {
    let state = ImeState::default().with_request(Some(
        ImeRequest::new(TextInputSessionId::new("editor"))
            .with_visibility(ImeVisibility::Visible)
            .with_selection(SelectionRange::new(1, 3)),
    ));

    set_ime_state(state.clone());
    assert_eq!(current_ime_state(), state);
}

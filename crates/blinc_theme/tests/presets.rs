use blinc_theme::{ColorScheme, ColorToken, RadiusToken, SpacingToken, ThemePreset, ThemeState};

#[test]
fn preset_catalog_contains_expected_presets() {
    let mut ids: Vec<&str> = ThemePreset::all().iter().map(|p| p.id()).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec!["blinc", "neutral", "slate", "zinc"]);
}

#[test]
fn shadcn_like_bundles_have_distinct_light_and_dark_primary() {
    for preset in [ThemePreset::Neutral, ThemePreset::Slate, ThemePreset::Zinc] {
        let bundle = preset.bundle();
        let light = bundle.for_scheme(ColorScheme::Light);
        let dark = bundle.for_scheme(ColorScheme::Dark);

        assert_ne!(
            light.colors().get(ColorToken::Primary),
            dark.colors().get(ColorToken::Primary),
            "Preset {:?} should have distinct light/dark primary colors",
            preset
        );
    }
}

#[test]
fn shadcn_like_presets_use_expected_radii() {
    for preset in [ThemePreset::Neutral, ThemePreset::Slate, ThemePreset::Zinc] {
        let bundle = preset.bundle();
        let light = bundle.for_scheme(ColorScheme::Light);

        assert_eq!(
            light.radii().get(RadiusToken::Md),
            10.0,
            "Preset {:?} should use md=10.0",
            preset
        );
        assert_eq!(
            light.radii().get(RadiusToken::Sm),
            6.0,
            "Preset {:?} should use sm=6.0",
            preset
        );
        assert_eq!(
            light.radii().get(RadiusToken::Lg),
            14.0,
            "Preset {:?} should use lg=14.0",
            preset
        );
    }
}

#[test]
fn shadcn_like_presets_use_readable_selection_text() {
    for preset in [ThemePreset::Neutral, ThemePreset::Slate, ThemePreset::Zinc] {
        let bundle = preset.bundle();
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            let theme = bundle.for_scheme(scheme);
            assert_eq!(
                theme.colors().get(ColorToken::SelectionText),
                theme.colors().get(ColorToken::TextPrimary),
                "preset={preset:?} scheme={scheme:?}"
            );
        }
    }
}

#[test]
fn shadcn_like_presets_expose_component_tokens_derived_from_primitive_scales() {
    for preset in [ThemePreset::Neutral, ThemePreset::Slate, ThemePreset::Zinc] {
        let bundle = preset.bundle();
        let theme = bundle.for_scheme(ColorScheme::Light);
        let components = theme.components();
        let spacing = theme.spacing();
        let radii = theme.radii();
        let typography = theme.typography();

        assert_eq!(components.control.height_md, spacing.space_10);
        assert_eq!(components.control.padding_x_md, spacing.space_4);
        assert_eq!(components.control.radius_md, radii.radius_default);
        assert_eq!(components.container.padding, spacing.space_6);
        assert_eq!(components.container.radius, radii.radius_xl);
        assert_eq!(components.overlay.radius, radii.radius_md);
        assert_eq!(components.typography.action_md, typography.text_sm);
        assert_eq!(components.typography.badge, typography.text_xs);
        assert_eq!(components.compact.cluster_gap_sm, spacing.space_1);
        assert_eq!(components.compact.progress_height_md, spacing.space_2);
    }
}

#[test]
fn theme_state_component_tokens_track_spacing_and_radius_overrides() {
    if ThemeState::try_get().is_none() {
        ThemeState::init(ThemePreset::Neutral.bundle(), ColorScheme::Light);
    }

    let theme = ThemeState::get();
    theme.clear_overrides();

    theme.set_spacing_override(SpacingToken::Space10, 52.0);
    theme.set_radius_override(RadiusToken::Default, 11.0);

    let components = theme.components();
    let vars = theme.to_css_variable_map();

    assert_eq!(components.control.height_md, 52.0);
    assert_eq!(components.control.radius_md, 11.0);
    assert_eq!(vars.get("control-height-md"), Some(&"52px".to_string()));
    assert_eq!(vars.get("control-radius-md"), Some(&"11px".to_string()));

    theme.remove_spacing_override(SpacingToken::Space10);
    theme.remove_radius_override(RadiusToken::Default);

    let reset_components = theme.components();
    let reset_vars = theme.to_css_variable_map();

    assert_eq!(reset_components.control.height_md, theme.spacing().space_10);
    assert_eq!(
        reset_components.control.radius_md,
        theme.radii().radius_default
    );
    assert_eq!(
        reset_vars.get("control-height-md"),
        Some(&format!("{}px", theme.spacing().space_10 as i32))
    );
}

#[test]
fn theme_preset_from_str_round_trips_with_id() {
    for preset in ThemePreset::all() {
        let parsed: ThemePreset = preset.id().parse().expect("preset id should parse");
        assert_eq!(parsed, *preset);
    }
}

#[test]
fn theme_preset_from_str_rejects_unknown_id() {
    let parsed: Result<ThemePreset, _> = "unknown".parse();
    assert!(parsed.is_err());
}

#[test]
fn theme_preset_serde_round_trip_uses_id_strings() {
    for preset in ThemePreset::all() {
        let json = serde_json::to_string(preset).expect("serialize preset");
        assert_eq!(json, format!("\"{}\"", preset.id()));

        let parsed: ThemePreset = serde_json::from_str(&json).expect("deserialize preset");
        assert_eq!(parsed, *preset);
    }
}

#[test]
fn theme_preset_serde_rejects_unknown_id() {
    let parsed: Result<ThemePreset, _> = serde_json::from_str("\"unknown\"");
    assert!(parsed.is_err());
}

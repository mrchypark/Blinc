use blinc_theme::{ColorScheme, ColorToken, RadiusToken, ThemePreset};

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

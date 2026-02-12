use blinc_theme::{ColorScheme, ColorToken, RadiusToken, ThemePreset};

#[test]
fn preset_catalog_contains_expected_presets() {
    let ids: Vec<&str> = ThemePreset::all().iter().map(|p| p.id()).collect();
    assert!(ids.contains(&"blinc"));
    assert!(ids.contains(&"neutral"));
    assert!(ids.contains(&"slate"));
    assert!(ids.contains(&"zinc"));
}

#[test]
fn neutral_bundle_has_distinct_light_and_dark_primary() {
    let bundle = ThemePreset::Neutral.bundle();
    let light = bundle.for_scheme(ColorScheme::Light);
    let dark = bundle.for_scheme(ColorScheme::Dark);

    assert_ne!(
        light.colors().get(ColorToken::Primary),
        dark.colors().get(ColorToken::Primary)
    );
}

#[test]
fn shadcn_like_presets_use_base_radius_ten_px() {
    let bundle = ThemePreset::Neutral.bundle();
    let light = bundle.for_scheme(ColorScheme::Light);

    assert_eq!(light.radii().get(RadiusToken::Md), 10.0);
    assert_eq!(light.radii().get(RadiusToken::Sm), 6.0);
    assert_eq!(light.radii().get(RadiusToken::Lg), 14.0);
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

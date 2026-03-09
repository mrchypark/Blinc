//! Semantic component tokens derived from primitive theme scales.

use crate::tokens::{RadiusTokens, Shadow, ShadowTokens, SpacingTokens, TypographyTokens};

/// Semantic defaults for control-like components such as buttons and inputs.
#[derive(Clone, Debug)]
pub struct ControlTokens {
    pub height_sm: f32,
    pub height_md: f32,
    pub height_lg: f32,
    pub padding_x_sm: f32,
    pub padding_x_md: f32,
    pub padding_x_lg: f32,
    pub padding_y_sm: f32,
    pub padding_y_md: f32,
    pub padding_y_lg: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
}

/// Semantic defaults for cards, dialogs, drawers, and other content containers.
#[derive(Clone, Debug)]
pub struct ContainerTokens {
    pub radius: f32,
    pub padding: f32,
    pub padding_compact: f32,
    pub header_gap: f32,
    pub footer_gap: f32,
    pub section_gap: f32,
}

/// Semantic defaults for menus, popovers, selects, and other floating overlays.
#[derive(Clone, Debug)]
pub struct OverlayTokens {
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub item_padding_x: f32,
    pub item_padding_y: f32,
    pub gap: f32,
    pub shadow: Shadow,
}

/// Semantic typography roles used by components.
#[derive(Clone, Debug)]
pub struct TypographyRoleTokens {
    pub action_sm: f32,
    pub action_md: f32,
    pub action_lg: f32,
    pub body_sm: f32,
    pub body_md: f32,
    pub body_lg: f32,
    pub label_sm: f32,
    pub label_md: f32,
    pub label_lg: f32,
    pub helper: f32,
    pub badge: f32,
    pub title: f32,
}

/// Semantic defaults for compact surfaces such as badges and keyboard hints.
#[derive(Clone, Debug)]
pub struct CompactTokens {
    pub badge_radius: f32,
    pub badge_padding_x: f32,
    pub badge_padding_y: f32,
    pub kbd_radius: f32,
    pub kbd_padding_x: f32,
    pub kbd_padding_y: f32,
    pub cluster_gap_sm: f32,
    pub cluster_gap_md: f32,
    pub progress_height_sm: f32,
    pub progress_height_md: f32,
    pub progress_height_lg: f32,
    pub switch_inset: f32,
}

/// Semantic component token set.
#[derive(Clone, Debug)]
pub struct ComponentTokens {
    pub control: ControlTokens,
    pub container: ContainerTokens,
    pub overlay: OverlayTokens,
    pub typography: TypographyRoleTokens,
    pub compact: CompactTokens,
}

impl ComponentTokens {
    /// Derive semantic component tokens from primitive scales.
    pub fn from_primitives(
        spacing: &SpacingTokens,
        radii: &RadiusTokens,
        typography: &TypographyTokens,
        shadows: &ShadowTokens,
    ) -> Self {
        Self {
            control: ControlTokens {
                height_sm: spacing.space_8,
                height_md: spacing.space_10,
                height_lg: spacing.space_12,
                padding_x_sm: spacing.space_3,
                padding_x_md: spacing.space_4,
                padding_x_lg: spacing.space_5,
                padding_y_sm: spacing.space_1,
                padding_y_md: spacing.space_2,
                padding_y_lg: spacing.space_3,
                radius_sm: radii.radius_sm,
                radius_md: radii.radius_default,
                radius_lg: radii.radius_md,
            },
            container: ContainerTokens {
                radius: radii.radius_xl,
                padding: spacing.space_6,
                padding_compact: spacing.space_4,
                header_gap: spacing.space_1_5,
                footer_gap: spacing.space_2,
                section_gap: spacing.space_4,
            },
            overlay: OverlayTokens {
                radius: radii.radius_md,
                padding_x: spacing.space_3,
                padding_y: spacing.space_2,
                item_padding_x: spacing.space_3,
                item_padding_y: spacing.space_2,
                gap: spacing.space_1,
                shadow: shadows.shadow_md.clone(),
            },
            typography: TypographyRoleTokens {
                action_sm: typography.text_xs + 1.0,
                action_md: typography.text_sm,
                action_lg: typography.text_base,
                body_sm: typography.text_xs + 1.0,
                body_md: typography.text_sm,
                body_lg: typography.text_base,
                label_sm: typography.text_xs,
                label_md: typography.text_sm,
                label_lg: typography.text_base,
                helper: typography.text_xs,
                badge: typography.text_xs,
                title: typography.text_lg,
            },
            compact: CompactTokens {
                badge_radius: radii.radius_full,
                badge_padding_x: spacing.space_2_5,
                badge_padding_y: spacing.space_0_5,
                kbd_radius: radii.radius_sm,
                kbd_padding_x: spacing.space_1_5,
                kbd_padding_y: spacing.space_0_5,
                cluster_gap_sm: spacing.space_1,
                cluster_gap_md: spacing.space_2,
                progress_height_sm: spacing.space_1,
                progress_height_md: spacing.space_2,
                progress_height_lg: spacing.space_3,
                switch_inset: spacing.space_0_5,
            },
        }
    }
}

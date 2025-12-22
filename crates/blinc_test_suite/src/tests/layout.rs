//! Layout system tests
//!
//! Tests for the GPUI-style layout builder API powered by Taffy flexbox.

use crate::runner::TestSuite;
use blinc_core::{Color, DrawContext};
use blinc_layout::prelude::*;

/// Create the layout test suite
pub fn suite() -> TestSuite {
    let mut suite = TestSuite::new("layout");

    // Basic flex row layout - three colored boxes in a row
    suite.add("flex_row_basic", |ctx| {
        let ui = div()
            .w(400.0)
            .h(120.0)
            .flex_row()
            .gap_px(20.0)
            .p_px(10.0)
            .bg(Color::rgba(0.15, 0.15, 0.2, 1.0))
            .child(div().w(100.0).h(100.0).rounded(8.0).bg(Color::rgba(0.9, 0.2, 0.3, 1.0))) // Red
            .child(div().w(100.0).h(100.0).rounded(8.0).bg(Color::rgba(0.2, 0.8, 0.3, 1.0))) // Green
            .child(div().w(100.0).h(100.0).rounded(8.0).bg(Color::rgba(0.2, 0.4, 0.9, 1.0))); // Blue

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(800.0, 600.0);
        tree.render(ctx.ctx());
    });

    // Flex column with gap - vertical stack
    suite.add("flex_col_with_gap", |ctx| {
        let ui = div()
            .w(150.0)
            .h(400.0)
            .flex_col()
            .gap_px(15.0)
            .p_px(15.0)
            .bg(Color::rgba(0.12, 0.12, 0.18, 1.0))
            .child(div().w_full().h(70.0).rounded(8.0).bg(Color::rgba(0.95, 0.3, 0.4, 1.0)))  // Coral
            .child(div().w_full().h(70.0).rounded(8.0).bg(Color::rgba(0.3, 0.85, 0.5, 1.0))) // Mint
            .child(div().w_full().h(70.0).rounded(8.0).bg(Color::rgba(0.3, 0.5, 0.95, 1.0))) // Sky blue
            .child(div().w_full().h(70.0).rounded(8.0).bg(Color::rgba(0.95, 0.85, 0.3, 1.0))); // Gold

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(800.0, 600.0);
        tree.render(ctx.ctx());
    });

    // Flex grow - fixed + flexible + fixed
    suite.add("flex_grow", |ctx| {
        let ui = div()
            .w(500.0)
            .h(100.0)
            .flex_row()
            .p_px(10.0)
            .gap_px(10.0)
            .bg(Color::rgba(0.1, 0.12, 0.18, 1.0))
            .child(div().w(80.0).h(80.0).rounded(8.0).bg(Color::rgba(0.9, 0.3, 0.4, 1.0)))  // Fixed left (coral)
            .child(div().flex_grow().h(80.0).rounded(8.0).bg(Color::rgba(0.3, 0.75, 0.6, 1.0))) // Flexible middle (teal)
            .child(div().w(80.0).h(80.0).rounded(8.0).bg(Color::rgba(0.5, 0.3, 0.9, 1.0))); // Fixed right (purple)

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(800.0, 600.0);
        tree.render(ctx.ctx());
    });

    // Nested layout - grid-like arrangement
    suite.add("nested_layout", |ctx| {
        let ui = div()
            .w(350.0)
            .h(350.0)
            .flex_col()
            .gap_px(12.0)
            .p_px(12.0)
            .rounded(16.0)
            .bg(Color::rgba(0.12, 0.13, 0.18, 1.0))
            // Top row: small square + expanding rectangle
            .child(
                div()
                    .w_full()
                    .h(90.0)
                    .flex_row()
                    .gap_px(12.0)
                    .child(div().w(90.0).h(90.0).rounded(12.0).bg(Color::rgba(0.95, 0.35, 0.45, 1.0))) // Coral
                    .child(div().flex_grow().h(90.0).rounded(12.0).bg(Color::rgba(1.0, 0.65, 0.3, 1.0))), // Orange
            )
            // Middle row: sidebar + main content
            .child(
                div()
                    .w_full()
                    .flex_grow()
                    .flex_row()
                    .gap_px(12.0)
                    .child(div().w(110.0).h_full().rounded(12.0).bg(Color::rgba(0.95, 0.85, 0.35, 1.0))) // Gold
                    .child(div().flex_grow().h_full().rounded(12.0).bg(Color::rgba(0.35, 0.85, 0.55, 1.0))), // Mint
            )
            // Bottom row: full width bar
            .child(div().w_full().h(60.0).rounded(12.0).bg(Color::rgba(0.35, 0.7, 0.95, 1.0))); // Sky blue

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(800.0, 600.0);
        tree.render(ctx.ctx());
    });

    // Justify content variations - four rows showing different alignments
    suite.add("justify_content", |ctx| {
        let c = ctx.ctx();

        // Helper colors for each row
        let bg_color = Color::rgba(0.18, 0.18, 0.24, 1.0);
        let box_colors = [
            Color::rgba(0.95, 0.35, 0.45, 1.0), // Coral
            Color::rgba(0.35, 0.85, 0.55, 1.0), // Mint
            Color::rgba(0.35, 0.55, 0.95, 1.0), // Blue
        ];

        // justify-start (default)
        let row1 = div()
            .w(380.0)
            .h(70.0)
            .flex_row()
            .p_px(5.0)
            .gap_px(10.0)
            .justify_start()
            .rounded(10.0)
            .bg(bg_color)
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[0]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[1]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[2]));

        let mut tree1 = RenderTree::from_element(&row1);
        tree1.compute_layout(800.0, 600.0);
        c.push_transform(blinc_core::Transform::translate(10.0, 10.0));
        tree1.render(c);
        c.pop_transform();

        // justify-center
        let row2 = div()
            .w(380.0)
            .h(70.0)
            .flex_row()
            .p_px(5.0)
            .gap_px(10.0)
            .justify_center()
            .rounded(10.0)
            .bg(bg_color)
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[0]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[1]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[2]));

        let mut tree2 = RenderTree::from_element(&row2);
        tree2.compute_layout(800.0, 600.0);
        c.push_transform(blinc_core::Transform::translate(10.0, 95.0));
        tree2.render(c);
        c.pop_transform();

        // justify-end
        let row3 = div()
            .w(380.0)
            .h(70.0)
            .flex_row()
            .p_px(5.0)
            .gap_px(10.0)
            .justify_end()
            .rounded(10.0)
            .bg(bg_color)
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[0]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[1]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[2]));

        let mut tree3 = RenderTree::from_element(&row3);
        tree3.compute_layout(800.0, 600.0);
        c.push_transform(blinc_core::Transform::translate(10.0, 180.0));
        tree3.render(c);
        c.pop_transform();

        // justify-between
        let row4 = div()
            .w(380.0)
            .h(70.0)
            .flex_row()
            .p_px(5.0)
            .justify_between()
            .rounded(10.0)
            .bg(bg_color)
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[0]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[1]))
            .child(div().w(60.0).h(60.0).rounded(8.0).bg(box_colors[2]));

        let mut tree4 = RenderTree::from_element(&row4);
        tree4.compute_layout(800.0, 600.0);
        c.push_transform(blinc_core::Transform::translate(10.0, 265.0));
        tree4.render(c);
        c.pop_transform();
    });

    // Padding test - outer and inner colors clearly visible
    suite.add("padding", |ctx| {
        let ui = div()
            .w(220.0)
            .h(220.0)
            .p_px(25.0)
            .rounded(16.0)
            .bg(Color::rgba(0.55, 0.25, 0.35, 1.0)) // Deep rose (outer padding area)
            .child(
                div()
                    .w_full()
                    .h_full()
                    .rounded(12.0)
                    .bg(Color::rgba(0.25, 0.55, 0.75, 1.0)), // Teal (inner content)
            );

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(800.0, 600.0);
        tree.render(ctx.ctx());
    });

    // Rounded corners with layout
    suite.add("rounded_layout", |ctx| {
        let ui = div()
            .w(320.0)
            .h(220.0)
            .p_px(18.0)
            .rounded(24.0)
            .bg(Color::rgba(0.14, 0.15, 0.22, 1.0)) // Dark slate
            .flex_col()
            .gap_px(14.0)
            .child(
                div()
                    .w_full()
                    .h(60.0)
                    .rounded(14.0)
                    .bg(Color::rgba(0.95, 0.4, 0.45, 1.0)), // Salmon
            )
            .child(
                div()
                    .w_full()
                    .flex_grow()
                    .rounded(14.0)
                    .bg(Color::rgba(0.3, 0.65, 0.95, 1.0)), // Bright blue
            );

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(800.0, 600.0);
        tree.render(ctx.ctx());
    });

    // Card-like component
    suite.add("card_component", |ctx| {
        let card = div()
            .w(280.0)
            .h(180.0)
            .p_px(16.0)
            .rounded(16.0)
            .bg(Color::rgba(0.95, 0.95, 0.97, 1.0))
            .flex_col()
            .gap_px(12.0)
            // Header row
            .child(
                div()
                    .w_full()
                    .h(40.0)
                    .flex_row()
                    .gap_px(12.0)
                    .items_center()
                    // Avatar placeholder
                    .child(div().square(40.0).rounded(20.0).bg(Color::rgba(0.3, 0.5, 0.9, 1.0)))
                    // Title area
                    .child(
                        div()
                            .flex_grow()
                            .h(40.0)
                            .flex_col()
                            .gap_px(4.0)
                            .child(div().w(120.0).h(14.0).rounded(3.0).bg(Color::rgba(0.2, 0.2, 0.25, 1.0)))
                            .child(div().w(80.0).h(10.0).rounded(2.0).bg(Color::rgba(0.6, 0.6, 0.65, 1.0))),
                    ),
            )
            // Content area
            .child(
                div()
                    .w_full()
                    .flex_grow()
                    .rounded(8.0)
                    .bg(Color::rgba(0.9, 0.9, 0.92, 1.0)),
            )
            // Button row
            .child(
                div()
                    .w_full()
                    .h(36.0)
                    .flex_row()
                    .justify_end()
                    .gap_px(8.0)
                    .child(
                        div()
                            .w(80.0)
                            .h(36.0)
                            .rounded(8.0)
                            .bg(Color::rgba(0.85, 0.85, 0.88, 1.0)),
                    )
                    .child(
                        div()
                            .w(80.0)
                            .h(36.0)
                            .rounded(8.0)
                            .bg(Color::rgba(0.3, 0.5, 0.9, 1.0)),
                    ),
            );

        let mut tree = RenderTree::from_element(&card);
        tree.compute_layout(800.0, 600.0);

        // Center the card
        ctx.ctx()
            .push_transform(blinc_core::Transform::translate(60.0, 60.0));
        tree.render(ctx.ctx());
        ctx.ctx().pop_transform();
    });

    // Glass card with layout - demonstrates glass over layout background
    // Uses the layout API with .effect() for glass and automatic foreground children
    suite.add_glass("glass_card", |ctx| {
        let c = ctx.ctx();

        // Colorful background shapes (will be blurred behind glass)
        c.fill_rect(
            blinc_core::Rect::new(0.0, 0.0, 400.0, 350.0),
            0.0.into(),
            Color::rgba(0.15, 0.2, 0.35, 1.0).into(),
        );

        // Colorful blobs for interesting blur effect
        c.fill_circle(
            blinc_core::Point::new(80.0, 80.0),
            70.0,
            Color::rgba(0.95, 0.35, 0.5, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(320.0, 120.0),
            80.0,
            Color::rgba(0.3, 0.85, 0.7, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(200.0, 280.0),
            60.0,
            Color::rgba(0.95, 0.75, 0.3, 1.0).into(),
        );
        c.fill_rect(
            blinc_core::Rect::new(30.0, 200.0, 100.0, 100.0),
            16.0.into(),
            Color::rgba(0.4, 0.5, 0.95, 1.0).into(),
        );

        // Build card with glass effect and foreground children (all via layout)
        // Wrap in container to position card at approximately (50, 75)
        let card_ui = div()
            .w(400.0)
            .h(350.0)
            .p_px(50.0)
            .child(
                div()
                    .w(300.0)
                    .h(200.0)
                    .mt(6.0) // Add ~25px top margin (6 * 4 = 24px)
                    .rounded(20.0)
                    .p_px(20.0)
                    .flex_col()
                    .gap_px(8.0)
                    .effect(
                        GlassMaterial::new()
                            .blur(25.0)
                            .tint_rgba(0.95, 0.95, 0.98, 0.6)
                            .saturation(1.1)
                            .border(1.0),
                    )
                    // Header bar - auto foreground
                    .child(
                        div()
                            .w(200.0)
                            .h(24.0)
                            .rounded(6.0)
                            .bg(Color::rgba(1.0, 1.0, 1.0, 0.9)),
                    )
                    // Subtitle - auto foreground
                    .child(
                        div()
                            .w(140.0)
                            .h(14.0)
                            .rounded(4.0)
                            .bg(Color::rgba(1.0, 1.0, 1.0, 0.6)),
                    )
                    // Content area - auto foreground
                    .child(
                        div()
                            .w(260.0)
                            .h(70.0)
                            .rounded(12.0)
                            .bg(Color::rgba(1.0, 1.0, 1.0, 0.15)),
                    )
                    // Button row - auto foreground
                    .child(
                        div()
                            .w_full()
                            .h(28.0)
                            .flex_row()
                            .justify_end()
                            .gap_px(10.0)
                            .child(
                                div()
                                    .w(60.0)
                                    .h(28.0)
                                    .rounded(8.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.2)),
                            )
                            .child(
                                div()
                                    .w(80.0)
                                    .h(28.0)
                                    .rounded(8.0)
                                    .bg(Color::rgba(0.3, 0.6, 0.95, 1.0)),
                            ),
                    ),
            );

        let mut tree = RenderTree::from_element(&card_ui);
        tree.compute_layout(400.0, 350.0);

        // Render with layer separation
        tree.render_to_layer(c, RenderLayer::Background);
        tree.render_to_layer(c, RenderLayer::Glass);

        let fg = ctx.foreground();
        tree.render_to_layer(fg, RenderLayer::Foreground);
    });

    // Layout-driven glass panel arrangement
    // Uses the layout API with .effect() for glass and automatic foreground children
    suite.add_glass("glass_layout_panels", |ctx| {
        let c = ctx.ctx();

        // Vibrant gradient-like background
        c.fill_rect(
            blinc_core::Rect::new(0.0, 0.0, 400.0, 350.0),
            0.0.into(),
            Color::rgba(0.2, 0.15, 0.35, 1.0).into(),
        );

        // Colorful shapes for blur effect
        c.fill_circle(
            blinc_core::Point::new(100.0, 100.0),
            90.0,
            Color::rgba(0.9, 0.3, 0.5, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(300.0, 250.0),
            100.0,
            Color::rgba(0.3, 0.8, 0.6, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(200.0, 50.0),
            70.0,
            Color::rgba(0.95, 0.7, 0.2, 1.0).into(),
        );

        // Build layout with glass panels and their children (all via layout)
        let layout = div()
            .w(360.0)
            .h(310.0)
            .flex_col()
            .gap_px(15.0)
            .p_px(20.0)
            // Top row: two panels side by side
            .child(
                div()
                    .w_full()
                    .h(120.0)
                    .flex_row()
                    .gap_px(15.0)
                    // Left panel with glass and content
                    .child(
                        div()
                            .w(140.0)
                            .h_full()
                            .rounded(16.0)
                            .p_px(15.0)
                            .flex_col()
                            .gap_px(10.0)
                            .effect(GlassMaterial::new().blur(20.0).tint_rgba(0.95, 0.95, 1.0, 0.5).border(0.8))
                            // Icon - auto foreground
                            .child(
                                div()
                                    .w(40.0)
                                    .h(40.0)
                                    .rounded(10.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.8)),
                            )
                            // Title - auto foreground
                            .child(
                                div()
                                    .w(110.0)
                                    .h(12.0)
                                    .rounded(4.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.6)),
                            )
                            // Subtitle - auto foreground
                            .child(
                                div()
                                    .w(80.0)
                                    .h(10.0)
                                    .rounded(3.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.4)),
                            ),
                    )
                    // Right panel with glass and content
                    .child(
                        div()
                            .flex_grow()
                            .h_full()
                            .rounded(16.0)
                            .p_px(15.0)
                            .flex_col()
                            .gap_px(10.0)
                            .effect(GlassMaterial::new().blur(20.0).tint_rgba(1.0, 0.95, 0.95, 0.5).border(0.8))
                            // Content area - auto foreground
                            .child(
                                div()
                                    .w_full()
                                    .h(60.0)
                                    .rounded(10.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.15)),
                            )
                            // Label - auto foreground
                            .child(
                                div()
                                    .w(100.0)
                                    .h(12.0)
                                    .rounded(4.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.5)),
                            ),
                    ),
            )
            // Bottom: full width panel with progress bar and controls
            .child(
                div()
                    .w_full()
                    .flex_grow()
                    .rounded(16.0)
                    .p_px(20.0)
                    .flex_col()
                    .gap_px(15.0)
                    .items_center()
                    .effect(GlassMaterial::new().blur(20.0).tint_rgba(0.95, 1.0, 0.95, 0.5).border(0.8))
                    // Progress bar container - auto foreground
                    .child(
                        div()
                            .w_full()
                            .h(8.0)
                            .rounded(4.0)
                            .bg(Color::rgba(1.0, 1.0, 1.0, 0.2)),
                    )
                    // Control buttons row - auto foreground
                    .child(
                        div()
                            .flex_row()
                            .gap_px(36.0)
                            .items_center()
                            // Previous button
                            .child(
                                div()
                                    .w(28.0)
                                    .h(28.0)
                                    .rounded(14.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.7)),
                            )
                            // Play button (larger)
                            .child(
                                div()
                                    .w(36.0)
                                    .h(36.0)
                                    .rounded(18.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.9)),
                            )
                            // Next button
                            .child(
                                div()
                                    .w(28.0)
                                    .h(28.0)
                                    .rounded(14.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.7)),
                            ),
                    ),
            );

        let mut tree = RenderTree::from_element(&layout);
        tree.compute_layout(400.0, 350.0);

        // Render with layer separation
        tree.render_to_layer(c, RenderLayer::Background);
        tree.render_to_layer(c, RenderLayer::Glass);

        let fg = ctx.foreground();
        tree.render_to_layer(fg, RenderLayer::Foreground);
    });

    // Material-based glass API test
    // This test demonstrates the new .glass() and .effect() methods on divs
    // Glass panels are automatically collected from the layout tree
    suite.add_glass("glass_material_api", |ctx| {
        let c = ctx.ctx();

        // Colorful background
        c.fill_rect(
            blinc_core::Rect::new(0.0, 0.0, 400.0, 350.0),
            0.0.into(),
            Color::rgba(0.12, 0.15, 0.25, 1.0).into(),
        );

        // Background shapes
        c.fill_circle(
            blinc_core::Point::new(100.0, 100.0),
            80.0,
            Color::rgba(0.9, 0.3, 0.4, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(300.0, 200.0),
            90.0,
            Color::rgba(0.3, 0.8, 0.5, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(150.0, 280.0),
            70.0,
            Color::rgba(0.3, 0.5, 0.95, 1.0).into(),
        );
        c.fill_circle(
            blinc_core::Point::new(280.0, 80.0),
            60.0,
            Color::rgba(0.95, 0.7, 0.2, 1.0).into(),
        );

        // Build layout with glass materials and children
        // This demonstrates:
        // - Glass materials via .effect()
        // - Children of glass elements are AUTOMATICALLY rendered in foreground
        // - No manual coordinate calculations or .foreground() calls needed!
        let ui = div()
            .w(360.0)
            .h(300.0)
            .p_px(20.0)
            .flex_col()
            .gap_px(15.0)
            // Top panel with glass material and children (auto foreground)
            .child(
                div()
                    .w_full()
                    .h(100.0)
                    .rounded(16.0)
                    .p_px(20.0)
                    .flex_col()
                    .gap_px(10.0)
                    .effect(
                        GlassMaterial::new()
                            .blur(25.0)
                            .tint_rgba(0.95, 0.95, 1.0, 0.5)
                            .border(1.0),
                    )
                    // Title bar - auto foreground as child of glass
                    .child(
                        div()
                            .w(180.0)
                            .h(16.0)
                            .rounded(4.0)
                            .bg(Color::rgba(1.0, 1.0, 1.0, 0.8)),
                    )
                    // Subtitle - auto foreground as child of glass
                    .child(
                        div()
                            .w(120.0)
                            .h(12.0)
                            .rounded(3.0)
                            .bg(Color::rgba(1.0, 1.0, 1.0, 0.5)),
                    ),
            )
            // Bottom row with two glass panels
            .child(
                div()
                    .w_full()
                    .flex_grow()
                    .flex_row()
                    .gap_px(15.0)
                    // Left panel with content
                    .child(
                        div()
                            .w(140.0)
                            .h_full()
                            .rounded(16.0)
                            .p_px(15.0)
                            .flex_col()
                            .gap_px(15.0)
                            .effect(GlassMaterial::thick().tint_rgba(1.0, 0.9, 0.9, 0.4))
                            // Icon placeholder - auto foreground
                            .child(
                                div()
                                    .w(50.0)
                                    .h(50.0)
                                    .rounded(12.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.7)),
                            )
                            // Label - auto foreground
                            .child(
                                div()
                                    .w(110.0)
                                    .h(10.0)
                                    .rounded(3.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.5)),
                            ),
                    )
                    // Right panel with content
                    .child(
                        div()
                            .flex_grow()
                            .h_full()
                            .rounded(16.0)
                            .p_px(15.0)
                            .flex_col()
                            .gap_px(15.0)
                            .items_center()
                            .effect(GlassMaterial::frosted().tint_rgba(0.9, 1.0, 0.9, 0.4))
                            // Content area - auto foreground
                            .child(
                                div()
                                    .w(140.0)
                                    .h(50.0)
                                    .rounded(10.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.15)),
                            )
                            // Button - auto foreground
                            .child(
                                div()
                                    .w(40.0)
                                    .h(40.0)
                                    .rounded(20.0)
                                    .bg(Color::rgba(1.0, 1.0, 1.0, 0.8)),
                            ),
                    ),
            );

        let mut tree = RenderTree::from_element(&ui);
        tree.compute_layout(400.0, 350.0);

        // Render the layout tree with layer separation:
        // - Background goes to background context
        // - Glass goes to background context (GPU renderer separates glass primitives)
        // - Foreground children of glass go to foreground context
        tree.render_to_layer(c, RenderLayer::Background);
        tree.render_to_layer(c, RenderLayer::Glass);

        // Render foreground to separate context (for proper glass compositing)
        let fg = ctx.foreground();
        tree.render_to_layer(fg, RenderLayer::Foreground);
    });

    suite
}

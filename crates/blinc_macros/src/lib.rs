//! Blinc procedural macros
//!
//! Provides derive macros for the Blinc UI framework.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Check if a field has the #[animation] attribute
fn has_animation_attr(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("animation"))
}

/// Derive macro that generates a unique compile-time key for a component
/// and generates field accessors based on field attributes.
///
/// This enables type-safe access to animation and state hooks without manual string keys.
/// The key is generated from the full module path and struct name, ensuring
/// uniqueness across the codebase.
///
/// # Field Attributes
///
/// - `#[animation]` - Field generates `use_<field_name>(ctx, initial, config)` returning `SharedAnimatedValue`
/// - No attribute - Field generates `use_<field_name>(ctx, initial)` returning `State<FieldType>`
///
/// # Example - Unit Struct (simple component key)
///
/// ```ignore
/// use blinc_macros::BlincComponent;
///
/// #[derive(BlincComponent)]
/// pub struct AnimatedDemoCard;
///
/// // In your UI builder:
/// fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
///     let ball_x = AnimatedDemoCard::use_animated_value(ctx, 20.0, SpringConfig::wobbly());
///     // ...
/// }
/// ```
///
/// # Example - Struct with Mixed Fields
///
/// ```ignore
/// use blinc_macros::BlincComponent;
///
/// #[derive(BlincComponent)]
/// pub struct PullToRefresh {
///     #[animation]
///     content_offset: f32,  // -> use_content_offset(ctx, initial, config) -> SharedAnimatedValue
///     #[animation]
///     icon_scale: f32,      // -> use_icon_scale(ctx, initial, config) -> SharedAnimatedValue
///     #[animation]
///     icon_opacity: f32,    // -> use_icon_opacity(ctx, initial, config) -> SharedAnimatedValue
/// }
///
/// fn build_ui(ctx: &WindowedContext) -> impl ElementBuilder {
///     let offset = PullToRefresh::use_content_offset(ctx, 0.0, SpringConfig::wobbly());
///     let scale = PullToRefresh::use_icon_scale(ctx, 0.5, SpringConfig::snappy());
///     // ...
/// }
/// ```
///
/// # Example - State Values
///
/// ```ignore
/// #[derive(BlincComponent)]
/// pub struct Counter {
///     count: i32,           // -> use_count(ctx, initial) -> State<i32>
///     step: i32,            // -> use_step(ctx, initial) -> State<i32>
///     #[animation]
///     scale: f32,           // -> use_scale(ctx, initial, config) -> SharedAnimatedValue
/// }
/// ```
///
/// # Generated Code
///
/// For all structs, the macro generates:
/// - A `COMPONENT_KEY` constant containing the unique key
/// - `use_animated_value` / `use_animated_value_with` for ad-hoc spring animations
/// - `use_animated_timeline` / `use_animated_timeline_with` for timeline animations
///
/// For structs with named fields:
/// - Fields with `#[animation]`: `use_<field_name>(ctx, initial, config)` -> `SharedAnimatedValue`
/// - Fields without attribute: `use_<field_name>(ctx, initial)` -> `State<FieldType>`
#[proc_macro_derive(BlincComponent, attributes(animation))]
pub fn derive_blinc_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract named fields if present and generate appropriate methods
    let field_methods = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                fields
                    .named
                    .iter()
                    .map(|field| {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = &field.ty;
                        let method_name =
                            syn::Ident::new(&format!("use_{}", field_name), field_name.span());
                        let field_key = format!("{}", field_name);

                        if has_animation_attr(field) {
                            // #[animation] attribute -> SharedAnimatedValue
                            quote! {
                                /// Get a persisted animated value for this field.
                                ///
                                /// Returns a `SharedAnimatedValue` that is persisted across UI rebuilds.
                                pub fn #method_name(
                                    ctx: &blinc_app::windowed::WindowedContext,
                                    initial: f32,
                                    config: blinc_animation::SpringConfig,
                                ) -> blinc_app::windowed::SharedAnimatedValue {
                                    let key = format!("{}:{}", Self::COMPONENT_KEY, #field_key);
                                    ctx.use_animated_value_for(key, initial, config)
                                }
                            }
                        } else {
                            // No attribute -> State<T>
                            quote! {
                                /// Get a persisted state value for this field.
                                ///
                                /// Returns a `State<#field_type>` that is persisted across UI rebuilds.
                                pub fn #method_name(
                                    ctx: &blinc_app::windowed::WindowedContext,
                                    initial: #field_type,
                                ) -> blinc_app::windowed::State<#field_type> {
                                    let key = format!("{}:{}", Self::COMPONENT_KEY, #field_key);
                                    ctx.use_state_keyed(&key, || initial)
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            }
            Fields::Unnamed(_) => Vec::new(),
            Fields::Unit => Vec::new(),
        },
        _ => Vec::new(),
    };

    // We use module_path!() + stringify!() in the generated code for a unique key
    let expanded = quote! {
        impl #name {
            /// Unique compile-time key for this component
            pub const COMPONENT_KEY: &'static str = concat!(module_path!(), "::", stringify!(#name));

            /// Get a persisted animated value for this component.
            ///
            /// The value is uniquely identified by the component type.
            /// Multiple calls with the same component return the same animation.
            pub fn use_animated_value(
                ctx: &blinc_app::windowed::WindowedContext,
                initial: f32,
                config: blinc_animation::SpringConfig,
            ) -> blinc_app::windowed::SharedAnimatedValue {
                ctx.use_animated_value_for(Self::COMPONENT_KEY, initial, config)
            }

            /// Get a persisted animated value with a suffix for multiple values per component.
            ///
            /// Use this when a component needs multiple independent animated values.
            ///
            /// # Example
            ///
            /// ```ignore
            /// let x = MyComponent::use_animated_value_with(ctx, "x", 0.0, SpringConfig::default());
            /// let y = MyComponent::use_animated_value_with(ctx, "y", 0.0, SpringConfig::default());
            /// ```
            pub fn use_animated_value_with(
                ctx: &blinc_app::windowed::WindowedContext,
                suffix: &str,
                initial: f32,
                config: blinc_animation::SpringConfig,
            ) -> blinc_app::windowed::SharedAnimatedValue {
                let key = format!("{}:{}", Self::COMPONENT_KEY, suffix);
                ctx.use_animated_value_for(key, initial, config)
            }

            /// Get a persisted animated timeline for this component.
            ///
            /// The timeline is uniquely identified by the component type.
            /// Multiple calls with the same component return the same timeline.
            pub fn use_animated_timeline(
                ctx: &blinc_app::windowed::WindowedContext,
            ) -> blinc_app::windowed::SharedAnimatedTimeline {
                ctx.use_animated_timeline_for(Self::COMPONENT_KEY)
            }

            /// Get a persisted animated timeline with a suffix for multiple timelines per component.
            ///
            /// Use this when a component needs multiple independent timelines.
            pub fn use_animated_timeline_with(
                ctx: &blinc_app::windowed::WindowedContext,
                suffix: &str,
            ) -> blinc_app::windowed::SharedAnimatedTimeline {
                let key = format!("{}:{}", Self::COMPONENT_KEY, suffix);
                ctx.use_animated_timeline_for(key)
            }

            #(#field_methods)*
        }
    };

    TokenStream::from(expanded)
}

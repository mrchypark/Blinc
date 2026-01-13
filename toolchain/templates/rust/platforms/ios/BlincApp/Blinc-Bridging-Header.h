//
//  Blinc-Bridging-Header.h
//  {{project_name}}
//
//  C declarations for Swift interop with Blinc Rust library.
//

#ifndef Blinc_Bridging_Header_h
#define Blinc_Bridging_Header_h

#include <stdint.h>
#include <stdbool.h>

// Opaque pointer to the Blinc render context
typedef struct IOSRenderContext IOSRenderContext;

// =============================================================================
// Context Management
// =============================================================================

IOSRenderContext* blinc_create_context(uint32_t width, uint32_t height, double scale_factor);
void blinc_destroy_context(IOSRenderContext* ctx);

// =============================================================================
// Frame Loop
// =============================================================================

bool blinc_needs_render(IOSRenderContext* ctx);
bool blinc_tick_animations(IOSRenderContext* ctx);
void blinc_build_frame(IOSRenderContext* ctx);
void blinc_mark_dirty(IOSRenderContext* ctx);

// =============================================================================
// Size and Layout
// =============================================================================

void blinc_update_size(IOSRenderContext* ctx, uint32_t width, uint32_t height, double scale_factor);
float blinc_get_width(IOSRenderContext* ctx);
float blinc_get_height(IOSRenderContext* ctx);

// =============================================================================
// Input Handling
// =============================================================================

void blinc_handle_touch(IOSRenderContext* ctx, uint64_t touch_id, float x, float y, int32_t phase);
void blinc_set_focused(IOSRenderContext* ctx, bool focused);

// =============================================================================
// Native Bridge
// =============================================================================

typedef char* (*NativeCallFn)(const char* ns, const char* name, const char* args_json);
void blinc_set_native_call_fn(NativeCallFn call_fn);
bool blinc_native_bridge_is_ready(void);
void blinc_free_string(char* ptr);

#endif /* Blinc_Bridging_Header_h */

shader_type canvas_item;

// Add required screen texture uniform
uniform sampler2D SCREEN_TEXTURE : hint_screen_texture, filter_linear_mipmap;

// Display settings
uniform bool overlay = false;

// New Chromatic Aberration settings from the second shader
uniform float intensity: hint_range(0.0, 5.0, 0.1) = 1.0;
uniform float threshold: hint_range(0.0, 2.0, 0.1) = 1.0;

// Additional effects
uniform bool discolor = true;
uniform float warp_amount : hint_range(-5.0, 5.0) = 1.0;

// The rest of the shader code remains exactly the same from here...
// (All the functions and fragment shader code remain unchanged)

// Screen warping
vec2 warp(vec2 uv) {
    vec2 delta = uv - 0.5;
    float delta2 = dot(delta.xy, delta.xy);
    float delta4 = delta2 * delta2;
    return uv + delta * (delta4 * warp_amount);
}

// Helper functions for Chromatic Aberration (from the second shader)
float vector_angle(vec2 v) {
    if (abs(v.x) < 0.0001 && abs(v.y) < 0.0001) {
        return 0.0; // Default angle for zero vector
    }
    return atan(v.y, v.x);
}

vec2 rotate_vector(vec2 v, float cos_theta, float sin_theta) {
    return vec2(
        v.x * cos_theta - v.y * sin_theta,
        v.x * sin_theta + v.y * cos_theta
    );
}


void fragment() {
    // Get base UV and handle overlay
    vec2 uv = overlay ? warp(SCREEN_UV) : warp(UV);
    vec2 text_uv = uv;

    // --- Start of new Chromatic Aberration logic ---
    vec2 center = vec2(0.5);
    vec2 dir = text_uv - center;
    float angle = vector_angle(dir);
    float dist = 2.0 * length(dir);
    float effect = exp(intensity * (dist - threshold));

    vec2 r_displacement = rotate_vector(effect * intensity * vec2(-1.0, 0.0), cos(angle), sin(angle));
    vec2 g_displacement = rotate_vector(effect * intensity * vec2(0.0, 0.0), cos(angle), sin(angle));
    vec2 b_displacement = rotate_vector(effect * intensity * vec2(1.0, 0.0), cos(angle), sin(angle));

    vec4 text;

    text.r = texture(SCREEN_TEXTURE, fma(r_displacement, SCREEN_PIXEL_SIZE, text_uv)).r;
    text.g = texture(SCREEN_TEXTURE, fma(g_displacement, SCREEN_PIXEL_SIZE, text_uv)).g;
    text.b = texture(SCREEN_TEXTURE, fma(b_displacement, SCREEN_PIXEL_SIZE, text_uv)).b;
    text.a = 1.0; // Keep the alpha channel consistent
    // --- End of new Chromatic Aberration logic ---

    // Apply VHS discoloration
    if (discolor) {
        vec3 greyscale = vec3(dot(text.rgb, vec3(0.333)));
        text.rgb = mix(text.rgb, greyscale, 0.5);
        float midpoint = pow(0.5, 2.2);
        text.rgb = (text.rgb - vec3(midpoint)) * 1.2 + midpoint;
    }

    COLOR = text;
}
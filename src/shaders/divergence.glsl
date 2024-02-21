precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_r_half_texel_size;
uniform vec2 u_resolution;
uniform sampler2D u_velocity;

void main() {
    vec2 l = v_uv - vec2(1.0, 0.0) / u_resolution;
    vec2 r = v_uv + vec2(1.0, 0.0) / u_resolution;
    vec2 b = v_uv - vec2(0.0, 1.0) / u_resolution;
    vec2 t = v_uv + vec2(0.0, 1.0) / u_resolution;

    float x_l = texture2D(u_velocity, l).x;
    float x_r = texture2D(u_velocity, r).x;
    float x_b = texture2D(u_velocity, b).y;
    float x_t = texture2D(u_velocity, t).y;
    vec2 x_c = texture2D(u_velocity, v_uv).xy;
    
    if (gl_FragCoord.x < 1.0) { x_l = -x_c.x; }
    else if (gl_FragCoord.x > u_resolution.x - 1.0) { x_r = -x_c.x; }

    if (gl_FragCoord.y < 1.0) { x_b = -x_c.y; }
    else if (gl_FragCoord.y > u_resolution.y - 1.0) { x_t = -x_c.y; }

    float divergence = (x_r - x_l + x_t - x_b) * u_r_half_texel_size;
    gl_FragColor = vec4(divergence, 0.0, 0.0, 0.0);
} 
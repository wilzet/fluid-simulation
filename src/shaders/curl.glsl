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

    float x_l = texture2D(u_velocity, l).y;
    float x_r = texture2D(u_velocity, r).y;
    float x_b = texture2D(u_velocity, b).x;
    float x_t = texture2D(u_velocity, t).x;

    float curl = ((x_t - x_b) - (x_r - x_l)) * u_r_half_texel_size;
    gl_FragColor = vec4(curl, 0.0, 0.0, 0.0);
} 
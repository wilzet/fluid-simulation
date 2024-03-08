precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_r_half_texel_size;
uniform vec2 u_resolution;
uniform sampler2D u_velocity;
uniform sampler2D u_pressure;
uniform sampler2D u_obstacles;

void main() {
    float x_l = texture2D(u_pressure, (gl_FragCoord.xy - vec2(1.0, 0.0)) / u_resolution).x;
    float x_r = texture2D(u_pressure, (gl_FragCoord.xy + vec2(1.0, 0.0)) / u_resolution).x;
    float x_b = texture2D(u_pressure, (gl_FragCoord.xy - vec2(0.0, 1.0)) / u_resolution).x;
    float x_t = texture2D(u_pressure, (gl_FragCoord.xy + vec2(0.0, 1.0)) / u_resolution).x;

    vec2 velocity = texture2D(u_velocity, v_uv).xy;
    velocity -= vec2(x_r - x_l, x_t - x_b) * u_r_half_texel_size;
    float obstacle = texture2D(u_obstacles, v_uv).x;
    gl_FragColor = vec4(velocity * obstacle, 0.0, 0.0);
} 
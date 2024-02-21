precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_alpha;
uniform float u_r_beta;
uniform vec2 u_resolution;
uniform sampler2D u_x;
uniform sampler2D u_b;

void main() {
    vec2 l = v_uv - vec2(1.0, 0.0) / u_resolution;
    vec2 r = v_uv + vec2(1.0, 0.0) / u_resolution;
    vec2 b = v_uv - vec2(0.0, 1.0) / u_resolution;
    vec2 t = v_uv + vec2(0.0, 1.0) / u_resolution;

    vec4 x_l = texture2D(u_x, l);
    vec4 x_r = texture2D(u_x, r);
    vec4 x_b = texture2D(u_x, b);
    vec4 x_t = texture2D(u_x, t);
    vec4 x_c = texture2D(u_x, v_uv);
    
    vec4 bC = texture2D(u_b, v_uv);
    gl_FragColor = (x_l + x_r + x_b + x_t + u_alpha * bC) * u_r_beta;
} 
precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_curl_scale;
uniform float u_r_half_texel_size;
uniform vec2 u_resolution;
uniform sampler2D u_curl;
uniform sampler2D u_velocity;

void main() {
    vec2 l = v_uv - vec2(1.0, 0.0) / u_resolution;
    vec2 r = v_uv + vec2(1.0, 0.0) / u_resolution;
    vec2 b = v_uv - vec2(0.0, 1.0) / u_resolution;
    vec2 t = v_uv + vec2(0.0, 1.0) / u_resolution;

    float x_l = abs(texture2D(u_curl, l).x);
    float x_r = abs(texture2D(u_curl, r).x);
    float x_b = abs(texture2D(u_curl, b).x);
    float x_t = abs(texture2D(u_curl, t).x);
    float x_c = texture2D(u_curl, v_uv).x;

    // w = curl(u) => only z-component
    // v = norm(grad(abs(u)))
    // f = cross(v, w) => swap x- and y-component (and y-component *= -1)
    vec2 gradient = vec2(x_t - x_b, x_l - x_r) * u_r_half_texel_size;
    vec2 vorticity = gradient / max(length(gradient), 0.0001);
    vec2 force = vorticity * x_c * u_curl_scale;

    vec2 velocity = texture2D(u_velocity, v_uv).xy;
    gl_FragColor = vec4(velocity + force, 0.0, 0.0);
} 
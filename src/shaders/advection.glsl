precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_dissipation;
uniform float u_delta_time;
uniform vec2 u_resolution;
uniform sampler2D u_velocity;
uniform sampler2D u_quantity;

void main() {
    vec2 velocity = texture2D(u_velocity, v_uv).xy / u_resolution;
    vec2 position = v_uv - velocity * u_delta_time;
    gl_FragColor = texture2D(u_quantity, position) * u_dissipation;
}
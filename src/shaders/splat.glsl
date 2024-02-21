precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_scaled_radius;
uniform vec2 u_position;
uniform vec3 u_color;
uniform sampler2D u_texture;

void main() {
    vec3 color = texture2D(u_texture, v_uv).xyz;
    vec2 distance = gl_FragCoord.xy - u_position;
    color += u_color * exp(-dot(distance, distance) / u_scaled_radius);

    gl_FragColor = vec4(color, 1.0);
}
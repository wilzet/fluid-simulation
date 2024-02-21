precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform float u_factor;
uniform float u_offset;
uniform sampler2D u_texture;

void main() {
    gl_FragColor = texture2D(u_texture, v_uv) * u_factor + u_offset;
}
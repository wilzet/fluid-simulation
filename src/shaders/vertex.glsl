precision highp float;

attribute vec2 a_coordinates;
attribute vec2 a_uv;
varying vec2 v_uv;

void main() {
    gl_Position = vec4(a_coordinates, 0.0, 1.0);
    v_uv = a_uv;
}
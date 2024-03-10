precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform vec3 u_obstacle_color;
uniform sampler2D u_obstacles;
uniform sampler2D u_texture;

void main() {
    vec4 value = texture2D(u_texture, v_uv);
    float obstacle = texture2D(u_obstacles, v_uv).x;
    if (obstacle < 0.5) value = vec4(u_obstacle_color, value.w);
    gl_FragColor = value;
}
precision highp float;
precision highp sampler2D;

varying vec2 v_uv;

uniform bool u_is_circle;
uniform float u_scaled_radius_sqr;
uniform vec2 u_position;

void main() {
    vec2 distance = gl_FragCoord.xy - u_position;
    float obstacle = 1.0;
    if (u_is_circle)
        obstacle = step(u_scaled_radius_sqr, dot(distance, distance));
    else if (distance.x * distance.x < u_scaled_radius_sqr && distance.y * distance.y < u_scaled_radius_sqr)
        obstacle = 0.0;

    gl_FragColor = vec4(obstacle, 0.0, 0.0, 0.0);
}
pub const U_DISSIPATION: &str = "u_dissipation";
pub const U_DELTA_TIME: &str = "u_delta_time";
pub const U_RESOLUTION: &str = "u_resolution";
pub const U_VELOCITY: &str = "u_velocity";
pub const U_QUANTITY: &str = "u_quantity";
pub const ADVECTION_SHADER_SOURCE: &str = "
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
";

pub const U_FACTOR: &str = "u_factor";
pub const U_OFFSET: &str = "u_offset";
pub const U_TEXTURE: &str = "u_texture";
pub const COPY_SHADER_SOURCE: &str = "
	precision highp float;
	precision highp sampler2D;
	
	varying vec2 v_uv;
	
	uniform float u_factor;
	uniform float u_offset;
	uniform sampler2D u_texture;
	
	void main() {
	    gl_FragColor = texture2D(u_texture, v_uv) * u_factor + u_offset;
	}
";

pub const U_R_HALF_TEXEL_SIZE: &str = "u_r_half_texel_size";
pub const CURL_SHADER_SOURCE: &str = "
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
";

pub const DIVERGENCE_SHADER_SOURCE: &str = "
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
	
	    float x_l = texture2D(u_velocity, l).x;
	    float x_r = texture2D(u_velocity, r).x;
	    float x_b = texture2D(u_velocity, b).y;
	    float x_t = texture2D(u_velocity, t).y;
	    vec2 x_c = texture2D(u_velocity, v_uv).xy;
	    
	    if (gl_FragCoord.x < 1.0) { x_l = -x_c.x; }
	    else if (gl_FragCoord.x > u_resolution.x - 1.0) { x_r = -x_c.x; }
	
	    if (gl_FragCoord.y < 1.0) { x_b = -x_c.y; }
	    else if (gl_FragCoord.y > u_resolution.y - 1.0) { x_t = -x_c.y; }
	
	    float divergence = (x_r - x_l + x_t - x_b) * u_r_half_texel_size;
	    gl_FragColor = vec4(divergence, 0.0, 0.0, 0.0);
	} 
";

pub const U_PRESSURE: &str = "u_pressure";
pub const GRADIENT_SUBTRACT_SHADER_SOURCE: &str = "
	precision highp float;
	precision highp sampler2D;
	
	varying vec2 v_uv;
	
	uniform float u_r_half_texel_size;
	uniform vec2 u_resolution;
	uniform sampler2D u_velocity;
	uniform sampler2D u_pressure;
	
	void main() {
	    float x_l = texture2D(u_pressure, (gl_FragCoord.xy - vec2(1.0, 0.0)) / u_resolution).x;
	    float x_r = texture2D(u_pressure, (gl_FragCoord.xy + vec2(1.0, 0.0)) / u_resolution).x;
	    float x_b = texture2D(u_pressure, (gl_FragCoord.xy - vec2(0.0, 1.0)) / u_resolution).x;
	    float x_t = texture2D(u_pressure, (gl_FragCoord.xy + vec2(0.0, 1.0)) / u_resolution).x;
	
	    vec2 velocity = texture2D(u_velocity, v_uv).xy;
	    velocity -= vec2(x_r - x_l, x_t - x_b) * u_r_half_texel_size;
	    gl_FragColor = vec4(velocity, 0.0, 0.0);
	} 
";

pub const U_ALPHA: &str = "u_alpha";
pub const U_R_BETA: &str = "u_r_beta";
pub const U_X: &str = "u_x";
pub const U_B: &str = "u_b";
pub const JACOBI_SOLVER_SHADER_SOURCE: &str = "
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
";

pub const U_SCALED_RADIUS: &str = "u_scaled_radius";
pub const U_POSITION: &str = "u_position";
pub const U_COLOR: &str = "u_color";
pub const SPLAT_SHADER_SOURCE: &str = "
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
";

pub const VERTEX_SHADER_SOURCE: &str = "
	precision highp float;
	
	attribute vec2 a_coordinates;
	attribute vec2 a_uv;
	varying vec2 v_uv;
	
	void main() {
	    gl_Position = vec4(a_coordinates, 0.0, 1.0);
	    v_uv = a_uv;
	}
";

pub const U_CURL_SCALE: &str = "u_curl_scale";
pub const U_CURL: &str = "u_curl";
pub const VORTICITY_SHADER_SOURCE: &str = "
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
";
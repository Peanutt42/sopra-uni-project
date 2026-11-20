import sphynx_engine::pcn_vertex::PCN_Vertex;
import sphynx_engine::camera::Camera;
import sphynx_engine::light::Light;

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) world_position: vec3<f32>,
	@location(1) world_normal: vec3<f32>,
	@location(2) wave_height: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> light: Light;

struct Time {
	// x is the elapsed time in seconds, rest is padding
	elapsed: vec4<f32>,
}
@group(2) @binding(0)
var<uniform> time: Time;

const WAVE_AMPLITUDE_X: f32 = 0.3;
const WAVE_AMPLITUDE_Z: f32 = 0.2;
const WAVE_FREQUENCY_X: f32 = 0.35;
const WAVE_FREQUENCY_Z: f32 = 0.45;
const WAVE_SPEED_X: f32 = 1.2;
const WAVE_SPEED_Z: f32 = 0.9;

fn wave_height(position: vec2<f32>, elapsed: f32) -> f32 {
	let wave_x = sin(position.x * WAVE_FREQUENCY_X + elapsed * WAVE_SPEED_X);
	let wave_z = cos(position.y * WAVE_FREQUENCY_Z + elapsed * WAVE_SPEED_Z);
	return wave_x * WAVE_AMPLITUDE_X + wave_z * WAVE_AMPLITUDE_Z;
}

fn wave_normal(position: vec2<f32>, elapsed: f32) -> vec3<f32> {
	let dx = WAVE_AMPLITUDE_X * WAVE_FREQUENCY_X
        * cos(position.x * WAVE_FREQUENCY_X + elapsed * WAVE_SPEED_X);
	let dz = -WAVE_AMPLITUDE_Z * WAVE_FREQUENCY_Z
        * sin(position.y * WAVE_FREQUENCY_Z + elapsed * WAVE_SPEED_Z);
	return normalize(vec3<f32>(-dx, 1.0, -dz));
}

@vertex
fn vs_main(vertex: PCN_Vertex) -> VertexOutput {
	let elapsed = time.elapsed.x;
	let horizontal_position = vertex.position.xz;
	let height = wave_height(horizontal_position, elapsed);

	let world_position = vec3<f32>(
		vertex.position.x,
		vertex.position.y + height,
		vertex.position.z,
	);

	var output = VertexOutput();
	output.position = camera.proj_view * vec4<f32>(world_position, 1.0);
	output.world_position = world_position;
	output.world_normal = wave_normal(horizontal_position, elapsed);
	output.wave_height = height;
	return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
	let normal = normalize(input.world_normal);
	let light_direction = normalize(light.position - input.world_position);
	let view_direction = normalize(camera.position - input.world_position);
	let halfway_direction = normalize(light_direction + view_direction);

	let diffuse = max(dot(normal, light_direction), 0.0);
	let specular = pow(max(dot(normal, halfway_direction), 0.0), 64.0);
	let fresnel = pow(1.0 - max(dot(normal, view_direction), 0.0), 3.0);

	let deep_color = vec3<f32>(0.02, 0.12, 0.26);
	let shallow_color = vec3<f32>(0.05, 0.38, 0.5);
	let height_mix = clamp(input.wave_height * 1.25 + 0.5, 0.0, 1.0);
	let water_color = mix(deep_color, shallow_color, height_mix);

	var color = water_color * (0.35 + diffuse * 0.65);
	color += light.color * specular * 0.7;
	color += vec3<f32>(0.12, 0.25, 0.35) * fresnel * 0.5;

	// HDR tonemapping
	color = color / (color + vec3<f32>(1.0));

	return vec4<f32>(color, 1.0);
}

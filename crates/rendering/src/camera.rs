use cgmath::{Angle, Deg, InnerSpace, Matrix4, Point3, Rad, Vector3, perspective};
use serde::{Deserialize, Serialize};
use wgpu::SurfaceConfiguration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Projection {
	Perspective { fov: f32 },
	Orthographic { view_size: f32 },
}

/// represents the game's camera view
/// TODO: add roll
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
	pub position: Point3<f32>,
	pub yaw: Rad<f32>,
	pub pitch: Rad<f32>,
	pub projection: Projection,
	pub z_near: f32,
	pub z_far: f32,
}

impl Camera {
	const WORLD_UP: Vector3<f32> = Vector3 {
		x: 0.0,
		y: 1.0,
		z: 0.0,
	};

	pub fn perspective(
		position: Point3<f32>,
		yaw: Rad<f32>,
		pitch: Rad<f32>,
		fov: f32,
		z_near: f32,
		z_far: f32,
	) -> Self {
		Self {
			position,
			yaw,
			pitch,
			projection: Projection::Perspective { fov },
			z_near,
			z_far,
		}
	}

	pub fn orthographic(
		position: Point3<f32>,
		yaw: Rad<f32>,
		pitch: Rad<f32>,
		view_size: f32,
		z_near: f32,
		z_far: f32,
	) -> Self {
		Self {
			position,
			yaw,
			pitch,
			projection: Projection::Orthographic { view_size },
			z_near,
			z_far,
		}
	}

	pub fn set_view_size(&mut self, view_size: f32) {
		match &mut self.projection {
			Projection::Orthographic { view_size: size } => *size = view_size,
			Projection::Perspective { .. } => {
				debug_assert!(
					false,
					"view_size is only meaningful for orthographic cameras"
				);
			}
		}
	}

	pub fn with_view_size(mut self, view_size: f32) -> Self {
		self.set_view_size(view_size);
		self
	}

	/// combines projection and view matrix (look-at-matrix)
	pub fn get_projection_view_matrix(&self, aspect: f32) -> Matrix4<f32> {
		let proj = match self.projection {
			Projection::Perspective { fov } => {
				perspective(Deg(fov), aspect, self.z_near, self.z_far)
			}
			Projection::Orthographic { view_size } => {
				let half_height = view_size / 2.0;
				let half_width = half_height * aspect;
				let inv_depth = 1.0 / (self.z_far - self.z_near);

				// WebGPU clips to 0..1 depth; cgmath::ortho uses -1..1
				Matrix4::new(
					1.0 / half_width,
					0.0,
					0.0,
					0.0,
					0.0,
					1.0 / half_height,
					0.0,
					0.0,
					0.0,
					0.0,
					-inv_depth,
					0.0,
					0.0,
					0.0,
					-self.z_near * inv_depth,
					1.0,
				)
			}
		};

		let view = Matrix4::look_to_rh(
			self.position,
			self.get_forward_direction(),
			self.get_up_direction(),
		);

		proj * view
	}

	/// aspect ratio with safe fallback to 16/9 on invalid width/height (for ex. when window is
	/// minimized)
	pub fn get_aspect(width: f32, height: f32) -> f32 {
		let aspect = width / height;
		if aspect.is_finite() && aspect > 0.0 {
			aspect
		} else {
			16.0 / 9.0
		}
	}

	pub fn get_aspect_from_swapchain(swapchain_config: &SurfaceConfiguration) -> f32 {
		Self::get_aspect(
			swapchain_config.width as f32,
			swapchain_config.height as f32,
		)
	}

	pub fn get_forward_direction(&self) -> Vector3<f32> {
		let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
		let (pitch_sin, pitch_cos) = self.pitch.sin_cos();

		Vector3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos).normalize()
	}

	pub fn get_right_direction(&self) -> Vector3<f32> {
		let forward = self.get_forward_direction();

		// looking straight down/up makes the cross product with WORLD_UP degenerate
		if forward.y.abs() > 0.999 {
			return Vector3::new(1.0, 0.0, 0.0);
		}

		forward.cross(Self::WORLD_UP).normalize()
	}

	pub fn get_up_direction(&self) -> Vector3<f32> {
		self.get_right_direction()
			.cross(self.get_forward_direction())
			.normalize()
	}
}

impl Default for Camera {
	fn default() -> Self {
		Self::perspective(
			Point3::<f32>::new(0.0, 0.0, 0.0),
			Deg(-90.0).into(),
			Rad(0.0),
			90.0,
			0.1,
			1000.0,
		)
	}
}

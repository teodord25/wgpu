use glam::Vec3;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub distance: f32,
    pub yaw: f32,   // in radians
    pub pitch: f32, // in radians
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO, // Looking at the origin
            distance: 5.0,      // 5 units away from target
            yaw: 0.0,           // Facing along +Z by default
            pitch: 0.0,

            // Initial eye position will be computed each frame from yaw/pitch/distance
            eye: Vec3::new(0.0, 0.0, 5.0),
            up: Vec3::Y, // World "up" is positive Y
        }
    }
}

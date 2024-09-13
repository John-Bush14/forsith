use crate::vulkan::{devices::device::VkDevice, vertex::{VkBuffer, VkDeviceMemory}};

use crate::engine::math::{Vec3, Vector};

pub(self) use crate::engine::update_memory;

/// the camera, containing all the projection matrix + camera model matrix related items
///
/// #### projection matrix related settings:
///
///     fov: f32 (degrees)
///
///     near: f32
///
///     far: f32
///
/// #### camera model matrix related settings:
///
///     eye: [f32;3]
///
///     target: [f32;3]
///
/// target is relative to eye, so [1, 0, 0] will always make you look left
pub struct WorldView {
    fov: f32,
    near: f32,
    far: f32,
    eye: [f32;3],
    target: [f32;3],
    pub(crate) aspect: f32,
    pub(crate) changed: (bool, bool),
    view_matrix: [[f32;4];4],
    projection_matrix: [[f32;4];4],
    _up_vector: [f32;3],
    matrix_2d: [[f32;4];4],
    uniform_buffers_2d: Vec<(VkBuffer, VkDeviceMemory)>,
    uniform_buffers_3d: Vec<(VkBuffer, VkDeviceMemory)>
}

impl WorldView {
    /// returns a manual std::mem::zeroed()
    pub fn zero() -> WorldView {WorldView { fov: 0.0, near: 0.0, far: 0.0, eye: [0.0;3], target: [0.0;3], aspect: 0.0, changed: (false, false), view_matrix: [[0.0;4];4], projection_matrix: [[0.0;4];4], _up_vector: [0.0;3], matrix_2d: [[0.0;4];4], uniform_buffers_2d: vec!(), uniform_buffers_3d: vec!() }}
}

impl WorldView {
    /// returns the aspect ratio of the window (width/height)
    pub fn get_aspect(&self) -> f32 {return self.aspect}

    pub(crate) fn get_2d_uniform_buffers(&self) -> &Vec<(u64, u64)> {&self.uniform_buffers_2d}

    pub(crate) fn get_3d_uniform_buffers(&self) -> &Vec<(u64, u64)> {&self.uniform_buffers_3d}

    /// sets the pov
    pub fn set_fov(&mut self, fov: f32) {self.fov = fov; self.changed.1 = true}

    /// sets the near
    pub fn set_near(&mut self, near: f32) {self.near = near; self.changed.1 = true}

    /// sets the far
    pub fn set_far(&mut self, far: f32) {self.far = far; self.changed.1 = true}

    /// sets the eye location
    pub fn set_eye(&mut self, eye: [f32;3]) {self.eye = eye; self.changed.0 = true}

    /// moves the eye location (eye.pos += `movement`)
    pub fn move_eye(&mut self, movement: [f32;3]) {
        self.set_eye(self.eye.iter().zip(movement.iter())
            .map(|(&a, &b)| if !b.is_nan() {a + b} else {a}).collect::<Vec<f32>>()
            .try_into().unwrap())
    }

    /// moves the location relative to where the camera is looking (except the y-axis)
    ///
    /// for example if the camera is looking at [1, 0, 0], then `move_eye_local([0, 0, 1])` will
    /// move it 1 unit forward so 1 unit to the x-axis (+ [1, 0, 0])
    pub fn move_eye_local(&mut self, movement: [f32;3]) {
        if movement == [0.0, 0.0, 0.0] {return;}

        let target: [f32;3] = self.target.iter().zip(self.eye.iter()).map(|(x, y)| x + y).collect::<Vec<f32>>().try_into().unwrap();

        let direction: [f32;3] = target.iter().zip(self.eye.iter()).map(|(target, eye)| target - eye).collect::<Vec<f32>>().try_into().unwrap();

        let length = (direction[0] * direction[0] +  direction[2] * direction[2]).sqrt();

        let direction_normalized: [f32;3] = direction.iter().map(|x| x/length).collect::<Vec<f32>>().try_into().unwrap();

        let side_direction = [-direction_normalized[2], direction_normalized[0]];

        self.move_eye([
            movement[0] * side_direction[0] + movement[2] * direction_normalized[0],
            movement[1],
            movement[0] * side_direction[1] + movement[2] * direction_normalized[2],
        ]);
    }

    /// set target pos
    pub fn set_target(&mut self, target: [f32;3]) {self.target = target; self.changed.0 = true}

    /// move target pos (target.pos += `movement`)
    pub fn move_target(&mut self, movement: [f32;3]) {
        self.set_target(self.target.iter().zip(movement.iter())
            .map(|(a, b)| a + b).collect::<Vec<f32>>()
            .try_into().unwrap())
    }

    /// set's the target position with yaw and pitch (in degrees)
    pub fn set_target_yaw_pitch(&mut self, yaw_degrees: f32, pitch_degrees: f32) {
        let yaw = yaw_degrees.to_radians(); let pitch = pitch_degrees.to_radians();

        let (cp, sp) = (pitch.cos(), pitch.sin());

        let (cy, sy) = (yaw.cos(), yaw.sin());

        self.set_target([
            cp * cy,
            sp,
            cp * sy
        ])
    }

    pub(crate) fn update(&mut self, aspect: f32, device: VkDevice) {
        if self.changed.0 || self.changed.1 || self.aspect != aspect {
            for i in 0 .. self.uniform_buffers_2d.len() {
                update_memory(self.uniform_buffers_3d[i].1, device, (self.get_view_matrix(), self.get_projection_matrix(aspect)));
                update_memory(self.uniform_buffers_2d[i].1, device, (self.get_2d_camera_matrix(), aspect));
            }
        }
    }

    pub(crate) fn get_view_matrix(&mut self) -> [[f32;4];4] {
        if self.changed.0 {
            let eye = Vec3 {
                x: self.eye[0],
                y: self.eye[1],
                z: self.eye[2]
            };

            let target = Vec3 {
                x: self.target[0],
                y: self.target[1],
                z: self.target[2]
            };

            let up = Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0
            };


            let f = target.normalize();

            let s = f.cross(&up).normalize();

            let u = s.cross(&f);

            self.view_matrix = [
                [s.x, u.x, -f.x, 0.0],
                [s.y, u.y, -f.y, 0.0],
                [s.z, u.z, -f.z, 0.0],
                [-eye.dot(&s), -eye.dot(&u), eye.dot(&f), 1.0]
            ];


            self.changed.0 = false;
        }

        return self.view_matrix;
    }

    pub(crate) fn get_projection_matrix(&mut self, aspect: f32) -> [[f32;4];4] {
        if self.changed.1 || self.aspect != aspect {
            let fo = 1.0/(self.fov.to_radians()/2.0).tan();
            let fas = fo/aspect;
            let ne = self.near;
            let fa = self.far;

            self.aspect = aspect;

            self.projection_matrix = [
                [fas, 0.0, 0.0             , 0.0 ],
                [0.0, -fo, 0.0             , 0.0 ],
                [0.0, 0.0, -fa/(fa-ne)     , -1.0],
                [0.0, 0.0, -(fa*ne)/(fa-ne), 0.0 ]
            ];


            self.changed.1 = false;
        }


        return self.projection_matrix;
    }

    pub(crate) fn get_2d_camera_matrix(&mut self) -> [[f32;4];4] {
        if self.changed.0 {
            self.matrix_2d = [
                [1.0, 0.0, 0.0, self.eye[0]],
                [0.0, 1.0, 0.0, self.eye[1]],
                [0.0, 0.0, 1.0, self.eye[2]],
                [0.0, 0.0, 0.0, 1.0]
            ];

            self.changed.0 = false;
        }

        return self.matrix_2d;
    }

    pub(crate) fn new(
        eye: [f32;3],
        target: [f32;3],
        fov: f32,
        far: f32,
        near: f32,
        mut uniform_buffers: Vec<Vec<(VkBuffer, VkDeviceMemory)>>
    ) -> WorldView {
        return WorldView {
            eye,
            target,
            fov,
            near,
            far,
            aspect: 0.0,
            changed: (true, true),
            view_matrix: [[0f32;4];4],
            projection_matrix: [[0f32;4];4],
            _up_vector: [0.0, 1.0, 0.0],
            matrix_2d: [[0f32;4];4],
            uniform_buffers_2d: uniform_buffers.pop().unwrap(),
            uniform_buffers_3d: uniform_buffers.pop().unwrap()
        }
    }
}

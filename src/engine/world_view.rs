use cgmath::{Deg, Matrix4, Point3, Vector3};


pub struct worldView {
    fov: f32,
    near: f32,
    far: f32,
    eye: [f32;3],
    target: [f32;3],
    pub aspect: f32,
    pub changed: (bool, bool),
    view_matrix: [[f32;4];4],
    projection_matrix: [[f32;4];4],
    pub pitch_degrees: f32,
    pub yaw_degrees: f32
}


impl worldView {
    pub fn set_fov(&mut self, fov: f32) {self.fov = fov; self.changed.1 = true}

    pub fn set_near(&mut self, near: f32) {self.near = near; self.changed.1 = true}

    pub fn set_far(&mut self, far: f32) {self.far = far; self.changed.1 = true}

    pub fn set_eye(&mut self, eye: [f32;3]) {self.eye = eye; self.changed.0 = true}

    pub fn move_eye(&mut self, movement: [f32;3]) {
        self.set_eye(self.eye.iter().zip(movement.iter())
            .map(|(a, b)| a + b).collect::<Vec<f32>>()
            .try_into().unwrap())
    }

    pub fn move_eye_local(&mut self, movement: [f32;3]) {
        let direction: [f32;3] = self.target.iter().zip(self.eye.iter()).map(|(target, eye)| target - eye).collect::<Vec<f32>>().try_into().unwrap();

        let length = (direction[0] * direction[0] +  direction[2] * direction[2]).sqrt();

        let direction_normalized: [f32;3] = direction.iter().map(|x| x/length).collect::<Vec<f32>>().try_into().unwrap();

        let side_direction = [-direction_normalized[2], direction_normalized[0]];

        self.move_eye([
            movement[0] * side_direction[0] + movement[2] * direction_normalized[0], 
            movement[1],
            movement[0] * side_direction[1] + movement[2] * direction_normalized[2], 
        ]);
    }
    
    pub fn set_target(&mut self, target: [f32;3]) {self.target = target; self.changed.0 = true}

    pub fn move_target(&mut self, movement: [f32;3]) {
        self.set_target(self.target.iter().zip(movement.iter())
            .map(|(a, b)| a + b).collect::<Vec<f32>>()
            .try_into().unwrap())
    }

    pub fn set_target_yaw_pitch(&mut self) {
        let yaw = self.yaw_degrees.to_radians(); let pitch = self.pitch_degrees.to_radians();

        let (cp, sp) = (pitch.cos(), pitch.sin());

        let (cy, sy) = (yaw.cos(), yaw.sin());

        self.set_target([
            cp * cy,
            sp,
            cp * sy
        ])
    }

    pub fn get_view_matrix(&mut self) -> [[f32;4];4] {
        if self.changed.0 {
            let target: [f32;3] = self.target.iter().zip(self.eye.iter()).map(|(x, y)| x + y).collect::<Vec<f32>>().try_into().unwrap();

            self.view_matrix = Matrix4::look_at_rh(
                Point3::from(self.eye),
                Point3::from(target),
                Vector3::new(0.0, 1.0, 0.0),
            ).into();

            self.changed.0 = false;
        }

        return self.view_matrix;
    }

    pub fn get_projection_matrix(&mut self, aspect: f32) -> [[f32;4];4] {
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

    pub fn new(eye: [f32;3], target: [f32;3], fov: f32, far: f32, near: f32) -> worldView {
        return worldView {
            eye: eye,
            target: target,
            fov: fov,
            near: near,
            far: far,
            aspect: 0.0,
            changed: (true, true),
            view_matrix: [[0f32;4];4],
            projection_matrix: [[0f32;4];4],
            pitch_degrees: 0.0,
            yaw_degrees: -90.0
        }
    }
}

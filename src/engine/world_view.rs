use cgmath::{Matrix4, Point3, Vector3};


pub struct WorldView {
    fov: f32,
    near: f32,
    far: f32,
    eye: [f32;3],
    target: [f32;3],
    pub aspect: f32,
    pub changed: (bool, bool),
    view_matrix: [[f32;4];4],
    projection_matrix: [[f32;4];4],
    _up_vector: [f32;3],
    matrix_2d: [[f32;4];4]
}


impl WorldView {
    pub fn set_fov(&mut self, fov: f32) {self.fov = fov; self.changed.1 = true}

    pub fn set_near(&mut self, near: f32) {self.near = near; self.changed.1 = true}

    pub fn set_far(&mut self, far: f32) {self.far = far; self.changed.1 = true}

    pub fn set_eye(&mut self, eye: [f32;3]) {self.eye = eye; self.changed.0 = true}

    pub fn move_eye(&mut self, movement: [f32;3]) {
        self.set_eye(self.eye.iter().zip(movement.iter())
            .map(|(&a, &b)| if !b.is_nan() {a + b} else {a}).collect::<Vec<f32>>()
            .try_into().unwrap())
    }

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
    
    pub fn set_target(&mut self, target: [f32;3]) {self.target = target; self.changed.0 = true}

    pub fn move_target(&mut self, movement: [f32;3]) {
        self.set_target(self.target.iter().zip(movement.iter())
            .map(|(a, b)| a + b).collect::<Vec<f32>>()
            .try_into().unwrap())
    }

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

    pub fn get_2d_camera_matrix(&mut self) -> [[f32;4];4] {
        if self.changed.0 {
            let eye: [f32;3] = [self.eye[0], self.eye[1], 0.0];
            
            let target = [self.eye[0], self.eye[1], -1.0];

            self.matrix_2d = Matrix4::look_at_rh(
                Point3::from(eye),
                Point3::from(target),
                Vector3::new(0.0, 1.0, 0.0),
            ).into();

            self.changed.0 = false;
        }

        return self.matrix_2d;
    }

    pub fn new(eye: [f32;3], target: [f32;3], fov: f32, far: f32, near: f32) -> WorldView {
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
            matrix_2d: [[0f32;4];4]
        }
    }
}

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
    projection_matrix: [[f32;4];4]
}


impl worldView {
    pub fn set_fov(&mut self, fov: f32) {self.fov = fov; self.changed.1 = true}

    pub fn set_near(&mut self, near: f32) {self.near = near; self.changed.1 = true}

    pub fn set_far(&mut self, far: f32) {self.far = far; self.changed.1 = true}

    pub fn set_eye(&mut self, eye: [f32;3]) {self.eye = eye; self.changed.0 = true}
    
    pub fn set_target(&mut self, target: [f32;3]) {self.target = target; self.changed.0 = true}

    pub fn look_at(&mut self, rotation: [f32;3]) {
        let (xc, yc, zc) = (rotation[0].cos(), rotation[1].cos(), rotation[2].cos());

        let (xs, ys, zs) = (rotation[0].sin(), rotation[1].sin(), rotation[2].sin());

        let yszc = ys * zc;

        // Compute direction vector
        let target_x = yc * zc;
        let target_y = xs * yszc - xc * zs;
        let target_z = xc * yszc + xs * zs;
        self.set_target([target_x, target_y, target_z]);
    }

    pub fn get_view_matrix(&mut self) -> [[f32;4];4] {
        if self.changed.0 {
            self.view_matrix = Matrix4::look_at_rh(
                Point3::from(self.eye),
                Point3::from(self.target),
                Vector3::new(0.0, 1.0, 0.0),
            ).into();

            self.changed.0 = false;
        }

        return self.view_matrix;
    }

    pub fn get_projection_matrix(&mut self, aspect: f32) -> [[f32;4];4] {
        if self.changed.1 || self.aspect != aspect {
            let fo = 1.0/(self.fov.to_radians()/2.0).tan() / aspect;
            let ne = self.near;
            let fa = self.far;

            self.aspect = aspect;

            self.projection_matrix = [
                [fo , 0.0, 0.0             , 0.0 ],
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
            projection_matrix: [[0f32;4];4]
        }
    }
}

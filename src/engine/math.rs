use std::ops::{Add, Sub, Mul, Div};


pub(crate) trait Vector {
    fn dot(&self, other: &Self) -> f32;

    fn normalize(&self) -> Self;

    fn cross(&self, other: &Self) -> Self;

    fn sum(&self) -> f32;
}


#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {Vec2 {x: self.x - other.x, y: self.y - other.y}}
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {Vec2 {x: self.x + other.x, y: self.y + other.y}}
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, a: f32) -> Vec2 {Vec2 {x: self.x * a, y: self.y * a}}
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, a: f32) -> Vec2 {Vec2 {x: self.x / a, y: self.y / a}}
}

impl Vector for Vec2 {
    fn dot(&self, other: &Vec2) -> f32 {
        return self.x * other.x + self.y * other.y;
    }

    fn normalize(&self) -> Self {
        let sum = self.sum() as f32;

        return Vec2 {x: self.x/sum, y: self.y/sum};
    }

    fn cross(&self, _other: &Self) -> Self {
        unreachable!();
    }

    fn sum(&self) -> f32 {
        return self.x + self.y;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {Vec3 {x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}}
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {Vec3 {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}}
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, a: f32) -> Vec3 {Vec3 {x: self.x * a, y: self.y * a, z: self.z * a}}
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, a: f32) -> Vec3 {Vec3 {x: self.x / a, y: self.y / a, z: self.z / a}}
}

impl Vector for Vec3 {
    fn dot(&self, other: &Vec3) -> f32 {
        return self.x * other.x + self.y * other.y + self.z * other.z;
    }

    fn normalize(&self) -> Self {
        let sum = self.sum() as f32;

        return Vec3 {x: self.x/sum, y: self.y/sum, z: self.z/sum};
    }

    fn cross(&self, other: &Self) -> Self {
        return Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn sum(&self) -> f32 {
        return self.x + self.y + self.z;
    }
}

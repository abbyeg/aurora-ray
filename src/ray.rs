use glam::DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Ray {
    pub fn new(origin: DVec3, direction: DVec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + t * self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ray() {
        let r = Ray::new(DVec3::ZERO, DVec3::new(1., 3., -1.));
        assert_eq!(r.origin, DVec3::ZERO);
        assert_eq!(r.direction, DVec3::new(1., 3., -1.));
    }

    #[test]
    fn test_ray_at() {
        let r1 = Ray::new(DVec3::ZERO, DVec3::new(1., 1., 1.));
        let r2 = Ray::new(DVec3::ZERO, DVec3::ZERO);
        let r3 = Ray::new(DVec3::new(3.5, -2.2, 19.), DVec3::new(-20., 12., 42.68));
        assert_eq!(r1.at(2.), DVec3::new(2., 2., 2.));
        assert_eq!(r2.at(5.), DVec3::ZERO);
        assert_eq!(r3.at(-102.), DVec3::new(2043.5, -1226.2, -4334.36));
    }
}

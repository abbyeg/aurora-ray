use std::ops::Range;

use glam::DVec3;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
};

pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
    pub material: Material,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = ray.direction.dot(ray.direction);
        let h = ray.direction.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_disc = discriminant.sqrt();

        // find nearest root within range
        let mut root = (h - sqrt_disc) / a;
        if !interval.contains(&root) {
            root = (h + sqrt_disc) / a;
            if !interval.contains(&root) {
                return None;
            }
        }
        let t = root;
        let point = ray.at(t);
        let outward_normal = (point - self.center) / self.radius;

        Some(HitRecord::new(
            point,
            outward_normal,
            t,
            ray,
            self.material.clone(),
        ))
    }
}

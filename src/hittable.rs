use std::ops::Range;

use glam::DVec3;

use crate::{material::Material, ray::Ray};

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord>;
}

pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable + Sync>>,
}

impl HittableList {
    pub fn hit(&self, ray: &Ray, interval: Range<f64>) -> Option<HitRecord> {
        let (_closest_t, hit_record) =
            self.objects
                .iter()
                .fold((interval.end, None), |acc, object| {
                    if let Some(hit_rec) = object.hit(ray, interval.start..acc.0) {
                        // hit something
                        (hit_rec.t, Some(hit_rec))
                    } else {
                        acc
                    }
                });

        hit_record
    }
}

pub struct HitRecord {
    pub point: DVec3,
    pub outward_normal: DVec3,
    pub t: f64, // position along the ray
    pub front_face: bool,
    pub material: Material,
}

impl HitRecord {
    pub fn new(
        point: DVec3,
        mut outward_normal: DVec3,
        t: f64,
        ray: &Ray,
        material: Material,
    ) -> Self {
        let (normal, front_face) = HitRecord::calculate_face_normal(ray, &mut outward_normal);
        Self {
            point,
            outward_normal: normal,
            t,
            front_face,
            material,
        }
    }

    /// Calculates if the outward normal is front or back facing.
    /// Expects outward normal to be of unit length.
    /// Returns the normal negated if it is inside
    pub fn calculate_face_normal(ray: &Ray, outward_normal: &mut DVec3) -> (DVec3, bool) {
        if ray.direction.dot(*outward_normal) > 0.0 {
            // ray is inside
            return (-*outward_normal, false);
        }
        (*outward_normal, true)
    }
}

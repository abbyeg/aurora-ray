use std::ops::Neg;

use glam::DVec3;
use rand::Rng;

use crate::{hittable::HitRecord, ray::Ray};

/// Note - albedo is how much light is reflected.

#[derive(Copy, Clone)]
pub enum Material {
    ///   Diffuse reflectance. Can be implemented by either always scatter
    ///   and attenuating light according to reflectance R, or it can 
    ///   sometimes scatter with probability 1 - R with no attenuation,
    ///   and absorb any ray that isn't scattered. Or some combination. 
    /// 
    ///   This implementation always scatters.
    Lambertian { albedo: DVec3 },
    ///   Reflective material.
    ///   
    Metal { albedo: DVec3, fuzz: f64 },
    ///   Any clear material.
    Dielectric { refractive_index: f64 },
}

impl Material {
    pub fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(DVec3, Ray)> {
        match self {
            Material::Lambertian { albedo } => {
                let mut scatter_direction = hit_record.outward_normal + random_unit_vector();
                
                // avoid where result of scatter_direction is close to 0 to prevent infinites/NaNs
                if near_zero(&scatter_direction) {
                    scatter_direction = hit_record.outward_normal;
                }

                let scattered = Ray::new(hit_record.point, scatter_direction);

                Some((*albedo, scattered))
            }
            Material::Metal { albedo, fuzz } => {
                let mut reflected = reflect(&ray.direction, &hit_record.outward_normal);
                reflected = reflected.normalize() + (fuzz * random_unit_vector());
                let scattered = Ray::new(hit_record.point, reflected);
                if scattered.direction.dot(hit_record.outward_normal) > 0.0 {
                    return Some((*albedo, scattered));
                }
                None
            }
            Material::Dielectric { refractive_index } => {
                let mut rng = rand::thread_rng();
                let attenuation = DVec3::new(1.0, 1.0, 1.0);
                let ri = if hit_record.front_face {
                    1.0 / refractive_index
                } else {
                    *refractive_index
                };

                let unit_direction = ray.direction.normalize();
                let cos_theta = unit_direction.dot(hit_record.outward_normal).neg().min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let cannot_refract = ri * sin_theta > 1.0;
                let direction: DVec3;

                if cannot_refract || reflectance(cos_theta, ri) > rng.gen::<f64>() {
                    direction = reflect(&unit_direction, &hit_record.outward_normal);
                } else {
                    direction = refract(&unit_direction, &hit_record.outward_normal, ri);
                }

                let scattered = Ray::new(hit_record.point, direction);

                Some((attenuation, scattered))
            }
        }
    }
}

fn near_zero(v: &DVec3) -> bool {
    let s = 1e-8;

    (v.x.abs() < s) && (v.y.abs() < s) && (v.z.abs() < s)
}

fn reflect(v: &DVec3, n: &DVec3) -> DVec3 {
    v - (2.0 * v.dot(*n) * n)
}

fn refract(uv: &DVec3, n: &DVec3, etai_over_etat: f64) -> DVec3 {
    let cos_theta = -uv.dot(*n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

/// Schlick's approximation for reflectance based on 
/// the cosine of 
fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
    let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
    r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

/// Calculates a random vector on the unit sphere and normalizes it
/// to extend to the sphere surface.
/// Avoids cases where very small vector components round to 0 when squared
/// by rejection points that lie within a "black hole" around the center.
/// For f64, support values of 1e-160 and greater.
fn random_unit_vector() -> DVec3 {
    let mut rng = rand::thread_rng();

    loop {
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let z = rng.gen_range(-1.0..1.0);
        let v = DVec3::new(x, y, z);
        let len_sq = v.length_squared();
        if len_sq > 1e-160 && len_sq <= 1.0 {
            return v / len_sq.sqrt();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_scatter_lambertian() {
        let lambertian = Material::Lambertian {
            albedo: DVec3::new(1., 2., 1.),
        };
    }

    #[test]
    fn test_random_unit_vector() {
        let random_vec1 = random_unit_vector();
        let random_vec2 = random_unit_vector();
        let random_vec3 = random_unit_vector();
        assert_eq!(random_vec1.length(), 1.);
        assert_eq!(random_vec2.length(), 1.);
        assert_eq!(random_vec3.length(), 1.);
    }

    #[test]
    fn test_reflectance() {

    }
}

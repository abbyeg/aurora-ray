use glam::DVec3;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use itertools::Itertools;
use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use rayon::prelude::*;
use std::{
    cell::RefCell, f64::consts::PI, fs::File, hash::{DefaultHasher, Hash, Hasher}, io::{self, BufWriter, Write}, ptr, sync::Arc, thread, time::{
        SystemTime, 
        UNIX_EPOCH
    }
};

use crate::{fastrand::random_f64, fastrand::random_in_range, hittable::HittableList};
use crate::ray::Ray;

const MAX_VAL: u8 = 255;

pub struct CameraBuilder {
    pub aspect_ratio: Option<f64>,
    pub image_width: Option<u32>,
    pub samples_per_pixel: Option<u32>,
    pub max_depth: Option<u32>,
    pub vertical_fov: Option<f64>,
    pub look_from: Option<DVec3>,
    pub look_at: Option<DVec3>,
    pub v_up: Option<DVec3>,
    pub defocus_angle: Option<f64>,
    pub focus_dist: Option<f64>,
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self {
            aspect_ratio: None,
            image_width: None,
            samples_per_pixel: None,
            max_depth: None,
            vertical_fov: None,
            look_from: None,
            look_at: None,
            v_up: None,
            defocus_angle: None,
            focus_dist: None,
        }
    }

    pub fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    pub fn image_width(mut self, image_width: u32) -> Self {
        self.image_width = Some(image_width);
        self
    }

    pub fn samples_per_pixel(mut self, samples_per_pixel: u32) -> Self {
        self.samples_per_pixel = Some(samples_per_pixel);
        self
    }

    pub fn max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    pub fn vertical_fov(mut self, vertical_fov: f64) -> Self {
        self.vertical_fov = Some(vertical_fov);
        self
    }

    pub fn look_from(mut self, look_from: DVec3) -> Self {
        self.look_from = Some(look_from);
        self
    }

    pub fn look_at(mut self, look_at: DVec3) -> Self {
        self.look_at = Some(look_at);
        self
    }

    pub fn v_up(mut self, v_up: DVec3) -> Self {
        self.v_up = Some(v_up);
        self
    }

    pub fn defocus_angle(mut self, defocus_angle: f64) -> Self {
        self.defocus_angle = Some(defocus_angle);
        self
    }

    pub fn focus_dist(mut self, focus_dist: f64) -> Self {
        self.focus_dist = Some(focus_dist);
        self
    }

    pub fn build(self) -> Camera {
        let image_width = self.image_width.unwrap_or(400);
        let aspect_ratio = self.aspect_ratio.unwrap_or(16. / 9.);
        let samples_per_pixel = self.samples_per_pixel.unwrap_or(100);
        let max_depth = self.max_depth.unwrap_or(50);
        let vertical_fov = self.vertical_fov.unwrap_or(90.);
        let look_from = self.look_from.unwrap_or(DVec3::ZERO);
        let look_at = self.look_at.unwrap_or(DVec3::new(0., 0., -1.));
        let v_up = self.v_up.unwrap_or(DVec3::Y);
        let defocus_angle = self.defocus_angle.unwrap_or(0.);
        let focus_dist = self.focus_dist.unwrap_or(100.);

        Camera::initialize(
            image_width,
            aspect_ratio,
            samples_per_pixel,
            max_depth,
            vertical_fov,
            look_from,
            look_at,
            v_up,
            defocus_angle,
            focus_dist,
        )
    }
}

pub struct Camera {
    image_width: u32,
    samples_per_pixel: u32,
    pixel_samples_scale: f64,
    max_depth: u32,
    image_height: u32,
    camera_center: DVec3,
    pixel_delta_u: DVec3, // distance between each pixel horizontally
    pixel_delta_v: DVec3, // distance between each pixel vertically
    pixel_00_loc: DVec3,
    defocus_angle: f64,
    defocus_disk_u: DVec3,
    defocus_disk_v: DVec3,
}

impl Camera {
    fn initialize(
        image_width: u32,
        aspect_ratio: f64,
        samples_per_pixel: u32,
        max_depth: u32,
        vertical_fov: f64,
        look_from: DVec3,
        look_at: DVec3,
        v_up: DVec3,
        defocus_angle: f64,
        focus_dist: f64,
    ) -> Self {
        let pixel_samples_scale = 1. / samples_per_pixel as f64;
        let mut image_height = image_width as f64 / aspect_ratio;

        image_height = if image_height < 1. { 1.0 } else { image_height };

        let theta = degrees_to_radians(vertical_fov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;

        // We do not use the aspect ratio to calculate the width because the ratio is an idealized proportion
        // and the actual proportion is not always the same
        let viewport_width = image_width as f64 / image_height * viewport_height;
        let camera_center = look_from;

        // establish camera basis
        let w = (look_from - look_at).normalize();
        let u = v_up.cross(w);
        let v = w.cross(u);

        // Vectors across the horizontal and vertical edges of the viewport
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        // Calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height;

        let viewport_upper_left =
            camera_center - (focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculate camera defocus disk basis vectors
        let defocus_radius = focus_dist * (degrees_to_radians(defocus_angle) / 2.0).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            samples_per_pixel,
            pixel_samples_scale,
            max_depth,
            image_width,
            image_height: image_height as u32,
            camera_center,
            pixel_delta_u,
            pixel_delta_v,
            pixel_00_loc,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn render(
        &mut self,
        world: &HittableList,
        file_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(file_path)?;
        let mut buf_writer = BufWriter::new(file);
        self.write_ppm_header(&mut buf_writer)?;
        let size: u64 = self.image_height as u64 * self.image_width as u64;

        let bar = Arc::new(ProgressBar::new(size));
        
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)")
                .expect("Failed to set progress bar style")
                .progress_chars("██░"),
        );
        bar.inc(0);

        // render each pixel
        let pixels: Vec<DVec3> = (0..(self.image_height as u32))
            .into_par_iter()
            .enumerate()
            .map(|(i, y)| {
                let row: Vec<DVec3> = (0..(self.image_width as u32)).map(|x| {
                    let pixel_color: DVec3 = (0..self.samples_per_pixel)
                    .map(|_| {
                        let ray = self.get_ray(x, y);
                        self.color(&ray, self.max_depth, &world)
                    })
                    .sum();
                    
                    self.pixel_samples_scale * pixel_color
                })
                .collect();
                if i % 100 == 0 {  // Batch updates to reduce overhead
                    Arc::clone(&bar).inc(100 * self.image_width as u64);
                }
                row
            })
            .flat_map(|row| row.to_vec())
            .collect();

        // render each pixel
        // let pixels = (0..(self.image_height as u32))
        //     .cartesian_product(0..(self.image_width as u32))
        //     .collect::<Vec<(u32, u32)>>()
        //     .into_par_iter()
        //     .progress_with(bar.clone())
        //     .map(|(y, x)| {
        //         let pixel_color: DVec3 = (0..self.samples_per_pixel)
        //             .map(|_| {
        //                 let ray = self.get_ray(x, y);
        //                 self.color(&ray, self.max_depth, &world)
        //             })
        //             .sum();
        //         self.pixel_samples_scale * pixel_color
        //     })
        //     .collect::<Vec<DVec3>>();

        pixels.into_iter().for_each(|pixel| {
            self.write_color(pixel, &mut buf_writer)
                .expect("Failed to write pixel color.")
        });

        buf_writer.flush()?;

        println!("Finished processing in {:?}", bar.elapsed());

        Ok(())
    }

    fn get_ray(&self, x: u32, y: u32) -> Ray {
        let pixel_center =
            self.pixel_00_loc + (x as f64 * self.pixel_delta_u) + (y as f64 * self.pixel_delta_v);
        let pixel_center_offset = pixel_center + self.sample_square();

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.camera_center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_center_offset - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }

    /// Returns the vector to a random point in the
    /// [-.5,-.5]-[+.5,+.5] unit square.
    fn sample_square(&self) -> DVec3 {
        let rx = random_f64() - 0.5;
        let ry = random_f64() - 0.5;
        // println!("rx: {rx}, ry: {ry}");
        DVec3::new(rx, ry, 0.0)
    }

    fn write_color(&self, pixel_color: DVec3, writer: &mut BufWriter<File>) -> io::Result<()> {
        let r = self.linear_to_gamma(pixel_color.x);
        let g = self.linear_to_gamma(pixel_color.y);
        let b = self.linear_to_gamma(pixel_color.z);

        let adj_color = DVec3::new(
            r.clamp(0.000, 0.999),
            g.clamp(0.000, 0.999),
            b.clamp(0.000, 0.999),
        ) * MAX_VAL as f64;

        writeln!(writer, "{} {} {}", adj_color.x as u8, adj_color.y as u8, adj_color.z as u8)?;
        // writer.write(&[adj_color.x as u8, adj_color.y as u8, adj_color.z as u8])?;
        
        Ok(())
    }

    fn write_ppm_header(&mut self, writer: &mut BufWriter<File>) -> io::Result<()> {
        writeln!(writer, "P3")?;
        writeln!(writer, "{} {}", self.image_width, self.image_height)?;
        writeln!(writer, "{}", MAX_VAL)?;
        
        Ok(())
    }

    fn color(&self, ray: &Ray, depth: u32, world: &HittableList) -> DVec3 {
        if depth == 0 {
            return DVec3::ZERO;
        }

        if let Some(hit_record) = world.hit(ray, 0.001..f64::INFINITY) {
            if let Some((attenuation, scattered)) = hit_record.material.scatter(&ray, &hit_record) {
                return attenuation * self.color(&scattered, depth - 1, world);
            }
            return DVec3::ZERO;
        }

        // render background if we don't hit anything
        let unit_direction = ray.direction.normalize();
        let a = 0.5 * (unit_direction.y + 1.0);
        let white = DVec3::new(1.0, 1.0, 1.0);
        let blue = DVec3::new(0.5, 0.7, 1.0);
        lerp(a, white, blue)
    }

    fn defocus_disk_sample(&self) -> DVec3 {
        let p = random_in_unit_disk();
        self.camera_center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    /// Approximates gamma space by using 2.0 as it's easier than
    /// raising to a power of 1/2.2
    fn linear_to_gamma(&self, linear_component: f64) -> f64 {
        if linear_component > 0.0 {
            return linear_component.sqrt();
        }

        0.0
    }
}

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

pub fn random_in_unit_disk() -> DVec3 {
    loop {
        let p = DVec3::new(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), 0.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

fn lerp(a: f64, start: DVec3, end: DVec3) -> DVec3 {
    (1.0 - a) * start + a * end
}

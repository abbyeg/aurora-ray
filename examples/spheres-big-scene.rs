use std::io;

use glam::DVec3;
use rand::Rng;

use aurora::{
    camera::CameraBuilder, hittable::HittableList, material::Material, shapes::sphere::Sphere,
};

fn main() -> io::Result<()> {
    let ground_material = Material::Lambertian {
        albedo: DVec3::new(0.5, 0.5, 0.5),
    };
    let mut world = HittableList { objects: vec![] };
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: ground_material,
    }));
    let mut rng = rand::thread_rng();

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f64 = rng.gen();
            let center = DVec3::new(
                a as f64 + 0.9 * rng.gen::<f64>(),
                0.2,
                b as f64 + 0.9 * rng.gen::<f64>(),
            );

            if (center - DVec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let random_color_1 = DVec3::new(rng.gen(), rng.gen(), rng.gen());
                let random_color_2 = DVec3::new(rng.gen(), rng.gen(), rng.gen());
                let sphere_mat;

                if choose_mat < 0.8 {
                    let albedo = random_color_1 * random_color_2;
                    sphere_mat = Material::Lambertian { albedo };
                    world.objects.push(Box::new(Sphere {
                        center,
                        radius: 0.2,
                        material: sphere_mat,
                    }));
                } else if choose_mat < 0.95 {
                    let albedo = DVec3::new(
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                    );
                    let fuzz = rng.gen_range(0.0..0.5);
                    sphere_mat = Material::Metal { albedo, fuzz };
                    world.objects.push(Box::new(Sphere {
                        center,
                        radius: 0.2,
                        material: sphere_mat,
                    }));
                } else {
                    sphere_mat = Material::Dielectric {
                        refractive_index: 1.5,
                    };
                    world.objects.push(Box::new(Sphere {
                        center,
                        radius: 0.2,
                        material: sphere_mat,
                    }));
                }
            }
        }
    }

    let material1 = Material::Dielectric {
        refractive_index: 1.5,
    };
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(0.0, 1.0, 0.1),
        radius: 1.0,
        material: material1,
    }));

    let material2 = Material::Lambertian {
        albedo: DVec3::new(0.4, 0.2, 0.1),
    };
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(-4.0, 1.0, 0.1),
        radius: 1.0,
        material: material2,
    }));

    let material3 = Material::Metal {
        albedo: DVec3::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    };
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(4.0, 1.0, 0.0),
        radius: 1.0,
        material: material3,
    }));

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1200;
    let samples_per_pixel = 250;
    let max_depth = 25;

    let mut camera = CameraBuilder::new()
        .image_width(image_width)
        .aspect_ratio(aspect_ratio)
        .samples_per_pixel(samples_per_pixel)
        .max_depth(max_depth)
        .build();

    let _ = camera.render(&world, "output/spheres-big-scene.ppm".to_string());

    Ok(())
}

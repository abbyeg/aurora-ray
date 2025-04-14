#![feature(thread_id_value)]
use std::io;

use aurora::{
    camera::{Camera, CameraBuilder}, framebuffer::Framebuffer, hittable::HittableList, material::Material, shapes::sphere::Sphere
};
use glam::DVec3;
use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use winit::{
    application::ApplicationHandler, 
    dpi::LogicalSize, 
    event::WindowEvent, 
    event_loop::{ControlFlow, EventLoop}, 
    window::{Window, WindowAttributes}
};

fn big_scene() -> io::Result<()> {
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
        center: DVec3::new(0.0, 1.0, 0.0),
        radius: 1.0,
        material: material1,
    }));

    let material2 = Material::Lambertian {
        albedo: DVec3::new(0.4, 0.2, 0.1),
    };
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(-4.0, 1.0, 0.),
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

    let mut camera = CameraBuilder::new()
        .image_width(600)
        .aspect_ratio(16.0 / 9.0)
        .samples_per_pixel(200)
        .max_depth(50)
        .vertical_fov(20.)
        .look_from(DVec3::new(13., 2., 3.))
        .look_at(DVec3::new(0., 0., 0.))
        .defocus_angle(0.6)
        .focus_dist(10.)
        .v_up(DVec3::Y)
        .build();

    let _ = camera.render(&world, "spheres-big-scene.ppm".to_string());
    Ok(())
}

fn simple(image_width: u32, aspect_ratio: f64) -> io::Result<Vec<[u8; 4]>> {
    let ground_material = Material::Lambertian {
        albedo: DVec3::new(0.5, 0.5, 0.5),
    };
    let mut world = HittableList { objects: vec![] };

    let material2 = Material::Lambertian {
        albedo: DVec3::new(0.5, 0.5, 0.5),
    };
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(0., 0., -1.2),
        radius: 0.5,
        material: material2,
    }));
    world.objects.push(Box::new(Sphere {
        center: DVec3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: ground_material,
    }));

    let mut camera = CameraBuilder::new()
        .image_width(image_width)
        .aspect_ratio(aspect_ratio)
        .samples_per_pixel(100)
        .max_depth(20)
        .vertical_fov(20.)
        .look_from(DVec3::new(13., 2., 3.))
        .look_at(DVec3::new(0., 0., 0.))
        .v_up(DVec3::new(0., 1., 0.))
        // .defocus_angle(60.)
        .focus_dist(10.)
        .build();

    let pixels = camera.get_pixels(&world).unwrap();
    println!("Rendered ok");
    
    Ok(pixels)
}


#[derive(Default)]
struct App<'a> {
    camera: Camera, 
    pixels: Option<Pixels<'static>>,
    window: Option<Arc<Window>>,

    width: u32,
    height: u32
}

impl <'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = Window::default_attributes()
        .with_title("I can't fully remember")
        .with_inner_size(winit::dpi::LogicalSize::new(600.0, 337.0))
        .with_resizable(false);

        self.window = Some(event_loop.create_window(attributes).unwrap());

        self.pixels = Some({
            let surface_texture = SurfaceTexture::new(self.width, self.height, self.window.as_mut().unwrap());
            Pixels::new(self.width, self.height as u32, surface_texture).unwrap()
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}


fn main() -> io::Result<()> {
    let image_width = 600;
    let aspect_ratio = 16./9.;
    let image_height = image_width as f64 / aspect_ratio;

    let generated_pixels = simple(image_width, aspect_ratio)?;
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    
    let _ = event_loop.run_app(&mut app);

    

    // write to frame buffer
    for (i, pixel) in pixels.frame_mut().chunks_exact_mut(4).enumerate() {
        // let x = (i % image_width as usize) as i16;
        // let y = (i / image_height as usize) as i16;

        pixel.copy_from_slice(&generated_pixels[i]);
    }
    println!("Finished copying over files!");

    pixels.render().unwrap();
    


    Ok(())
}

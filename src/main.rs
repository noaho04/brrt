use glam::{vec3, Vec3};
use std::f32::consts::PI;
use std::fs::{self};
use std::io::Write;

const WIDTH:  usize = 1024;
const HEIGHT: usize = 768;
const FOV:    f32   = PI/2.0;

#[derive(Copy, Clone)]
struct Sphere {
    center: Vec3,
    radius: f32
}

impl Sphere {
    fn new(center: Vec3, radius: f32) -> Self {
        Sphere { center, radius }
    }

    fn ray_intersect(&self, orig: Vec3, dir: Vec3, mut t0: f32) -> bool {
        let oc = self.center - orig;
        let tca = oc.dot(dir);

        let d2 = oc.length_squared() - tca*tca;
        let r2 = self.radius*self.radius;
        if d2 > r2 { return false };

        let thc = (r2 - d2).sqrt();
        t0 = tca + thc;
        let t1 = tca + thc;
        if t0 < 0.0 { t0 = t1 };
        if t0 < 0.0 { return false };
        
        true
    }
}

fn cast_ray(orig: Vec3, dir: Vec3, sphere: Sphere) -> Vec3 {
    let sphere_dist = f32::MAX;
    if !sphere.ray_intersect(orig, dir, sphere_dist) { return vec3(0.2, 0.7, 0.8) };

    vec3(0.4, 0.4, -0.3)
}

fn render(sphere: Sphere) -> std::io::Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    write!(&mut buffer, "P6\n{WIDTH} {HEIGHT}\n255\n").unwrap();

    let mut framebuffer: Vec<Vec3> = vec![Vec3::ZERO; WIDTH*HEIGHT];

    for j in 0 .. HEIGHT {
        for i in 0 .. WIDTH {
            let x = (2.0 * ((i as f32) + 0.5)/(WIDTH as f32) - 1.0) * (FOV/2.0).tan()*(WIDTH as f32) / (HEIGHT as f32);
            let y = (2.0 * ((j as f32) + 0.5)/(HEIGHT as f32) - 1.0) * (FOV/2.0).tan();
            let dir = vec3(x, y, -1.0).normalize();
            framebuffer[i+j*WIDTH] = cast_ray(vec3(0.0,0.0,0.0), dir, sphere)
        }
    }

    for pixel in framebuffer.iter().take(WIDTH*HEIGHT) {
        for channel in 0 .. 3 {
            let val = (255.0 * pixel[channel].clamp(0.0, 1.0)) as u8;
            buffer.push(val);
        }
    }
    fs::write("image.ppm", buffer)?;

    Ok(())
}

fn main() {
    let sphere = Sphere::new(vec3(-3.0, -0.0, -16.0), 2.0);
    render(sphere).unwrap();
}

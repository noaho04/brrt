use glam::{Vec2, Vec3, Vec4, vec2, vec3, vec4};
use rayon::prelude::*;
use std::cmp::min_by;
use std::fs::{self};
use std::io::Write;
use std::mem;
use std::vec;

const WIDTH:   usize = 1024;
const HEIGHT:  usize = 768;
const FOV:       f32 = 1.0;
const MAX_DEPTH: u32 = 4;

struct Light {
    position: Vec3,
    intensity: f32
}

impl Light {
    fn new(p: Vec3, i: f32) -> Self {
        Self { position: p, intensity: i }
    }
}

#[derive(Copy, Clone)]
struct Material {
    refractive_index:  f32,
    albedo:            Vec4,
    diffuse_color:     Vec3,
    specular_exponent: f32
}

impl Default for Material {
    fn default() -> Self {
        Self {
            refractive_index: 1.0,
            albedo: vec4(1.0,0.0,0.0,0.0), 
            diffuse_color: Vec3::ZERO, 
            specular_exponent: 0.0 
        }
    }
}

impl Material {
    fn new(r_i: f32, a: Vec4, color: Vec3, spec: f32) -> Self {
        Self { refractive_index: r_i, albedo: a, diffuse_color: color, specular_exponent: spec }
    }
}

#[derive(Copy, Clone)]
struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material
}

impl Sphere {
    fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Self { center, radius, material }
    }

    fn ray_intersect(&self, orig: Vec3, dir: Vec3, t0: &mut f32) -> bool {
        let oc = self.center - orig;
        let tca = oc.dot(dir);

        let d2 = oc.length_squared() - tca*tca;
        let r2 = self.radius*self.radius;
        if d2 > r2 { return false };

        let thc = (r2 - d2).sqrt();
        *t0 = tca - thc;
        let t1 = tca + thc;
        if *t0 < 0.0 { *t0 = t1 };
        if *t0 < 0.0 { return false };
        
        true
    }
}

fn reflect(i: Vec3, n: Vec3) -> Vec3 {
    i - 2.0 * i.dot(n) * n
}

fn refract(i: Vec3, n: Vec3, refractive_index: f32) -> Vec3 {
    let mut cosi = - i.dot(n).clamp(-1.0, 1.0);
    let mut etai = 1.0;
    let mut etat = refractive_index;
    let mut m = n;
    if (cosi < 0.0) {
        cosi = -cosi;
        mem::swap(&mut etai, &mut etat);
        m = -n;
    }
    let eta = etai / etat;
    let k = 1.0 - eta*eta*(1.0 - cosi*cosi);

    if (k < 0.0) { Vec3::ZERO } else { i*eta + m*(eta * cosi - k.sqrt()) }
}

fn scene_intersect(orig: Vec3, dir: Vec3, spheres: &Vec<Sphere>, hit: &mut Vec3, n: &mut Vec3, material: &mut Material) -> bool {
    let mut spheres_dist = f32::MAX;
    for sphere in spheres {
        let mut dist_i = 0.0;
        if sphere.ray_intersect(orig, dir, &mut dist_i) && dist_i < spheres_dist {
            spheres_dist = dist_i;
            *hit = orig + dir*dist_i;
            *n = (*hit - sphere.center).normalize();
            *material = sphere.material;
        }
    }

    let mut checkerboard_dist = f32::MAX;
    if (dir.y.abs() > 1e-3) {
        let d = -(orig.y+4.0)/dir.y;
        let pt = orig + dir * d;
        if (d>0.0 && pt.x.abs()<10.0 && pt.z<(-10.0) && pt.z>-30.0 && d<spheres_dist) {
            checkerboard_dist = d;
            *hit = pt;
            *n = vec3(0.0,1.0,0.0);
            let checked = ((hit.x * 0.5).floor() as i32
                         ^ (hit.z * 0.5).floor() as i32) & 1 != 0;
            material.diffuse_color = if checked {
                Vec3::splat(0.3)
            } else {
                vec3(0.3,0.2,0.1)
            }
        }
    }
    spheres_dist.min(checkerboard_dist) < 1000.0
}

fn cast_ray(orig: Vec3, dir: Vec3, spheres: &Vec<Sphere>, lights: &Vec<Light>, depth: Option<u32>) -> Vec3 {
    let current_depth = depth.unwrap_or(0);

    let mut point    = Vec3::default();
    let mut n        = Vec3::default();
    let mut material = Material::default();

    if current_depth > 4 || !scene_intersect(orig, dir, spheres, &mut point, &mut n, &mut material) {
        return vec3(0.2, 0.7, 0.8);
    }

    let reflect_dir = reflect(dir, n).normalize();
    let refract_dir = refract(dir, n, material.refractive_index).normalize();
    let reflect_orig = if reflect_dir.dot(n) < 0.0 { point - n*1e-3 } else { point + n*1e-3 };
    let refract_orig = if refract_dir.dot(n) < 0.0 { point - n*1e-3 } else { point + n*1e-3 };
    let reflect_color = cast_ray(reflect_orig, reflect_dir, spheres, lights, Some(current_depth + 1));
    let refract_color = cast_ray(refract_orig, refract_dir, spheres, lights, Some(current_depth + 1));

    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    for light in lights {
        let light_vec = light.position - point;
        let light_dir = light_vec.normalize();
        let light_dist = light_vec.length();

        let shadow_orig = if light_dir.dot(n) < 0.0 { point - n*1e-3 } else { point + n*1e-3 };
        let mut shadow_pt = Vec3::ZERO;
        let mut shadow_n  = Vec3::ZERO;
        let mut tmp_material = Material::default();
        if scene_intersect(shadow_orig, light_dir, spheres, &mut shadow_pt, &mut shadow_n, &mut tmp_material) { continue; }

        diffuse_light_intensity += light.intensity * light_dir.dot(n).max(0.0);
        let reflect_dir = reflect(light_dir, n);
        specular_light_intensity += reflect_dir
            .dot(dir)
            .max(0.0)
            .powf(material.specular_exponent)
            * light.intensity;
    }

    material.diffuse_color 
    * diffuse_light_intensity 
    * material.albedo[0] 
    + Vec3::ONE 
    * specular_light_intensity 
    * material.albedo[1]
    + reflect_color*material.albedo[2]
    + refract_color*material.albedo[3]
}

fn render(spheres: &Vec<Sphere>, lights: &Vec<Light>) -> std::io::Result<()> {
    let mut buffer = Vec::with_capacity(WIDTH*HEIGHT*3 + 32);
    write!(&mut buffer, "P6\n{WIDTH} {HEIGHT}\n255\n").unwrap();

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            let x = (2.0 * ((i as f32) + 0.5)/(WIDTH as f32) - 1.0) 
                    * (FOV/2.0).tan()
                    * (WIDTH as f32) / (HEIGHT as f32);

            let y = -(2.0 * ((j as f32) + 0.5)/(HEIGHT as f32) - 1.0) 
                    * (FOV/2.0).tan();

            let dir = vec3(x, y, -1.0).normalize();

            let pixel = cast_ray(Vec3::ZERO, dir, spheres, lights, None);

            let max = pixel.max_element().max(1.0);
            let color = pixel / max;

            for channel in 0..3 {
                buffer.push((255.0 * color[channel].clamp(0.0, 1.0)) as u8);
            }
        }
    }

    fs::write("image.ppm", buffer)
}

fn main() {
    let ivory      = Material::new(1.0, vec4(0.6, 0.3,  0.1, 0.0), vec3(0.4, 0.4, 0.3), 50.0);
    let red_rubber = Material::new(1.0, vec4(0.9, 0.1,  0.0, 0.0), vec3(0.3, 0.1, 0.1), 10.0);
    let glass      = Material::new(1.5, vec4(0.0, 0.5,  0.1, 0.8), vec3(0.6, 0.7, 0.8), 125.0);
    let mirror     = Material::new(1.0, vec4(0.0, 10.0, 0.8, 0.0), vec3(1.0, 1.0, 1.0), 1425.0);

    let mut spheres: Vec<Sphere> = vec![
        Sphere::new(vec3(-3.0,  0.0, -16.0), 2.0, ivory),
        Sphere::new(vec3(-1.0, -1.5, -12.0), 2.0, glass),
        Sphere::new(vec3( 1.5, -0.5, -18.0), 3.0, red_rubber),
        Sphere::new(vec3( 7.0,  5.0, -18.0), 4.0, mirror)
    ];

    let mut lights: Vec<Light> = vec![
        Light::new(vec3(-20.0, 20.0,  20.0), 1.5),
        Light::new(vec3( 30.0, 50.0, -25.0), 1.8),
        Light::new(vec3( 30.0, 20.0,  30.0), 1.7)
    ];

    render(&spheres, &lights);
}


//use cgmath::prelude::*;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;

#[derive(Debug)]
pub struct MousePicker {
    
}

impl MousePicker{
    pub fn GetModelCoordinatesForVoxelUnderMouse( window_size: &winit::dpi::PhysicalSize<u32>, mouse_device_coord: &winit::dpi::PhysicalPosition<f64>, 
                                                camera: &crate::camera::Camera, projection: &crate::camera::Projection, model: &crate::model::Model) -> cgmath::Vector3<f32>
    {
        //https://antongerdelan.net/opengl/raycasting.html
        // Step 1: 3d Normalised Device Coordinates
        let x = (2.0 * mouse_device_coord.x) / window_size.width as f64 - 1.0;
        let y = 1.0 - (2.0 * mouse_device_coord.y) / window_size.height as f64;
        let z = 1.0;
        let ray_nds : cgmath::Vector3<f32> = cgmath::Vector3::new(x as f32, y as f32, z);

        //Step 2: 4d Homogeneous Clip Coordinates
        let ray_clip : cgmath::Vector4<f32> = cgmath::Vector4::new(ray_nds.x, ray_nds.y, -1.0, 1.0);

        //Step 3: 4d Eye (Camera) Coordinates
        //vec4 ray_eye = inverse(projection_matrix) * ray_clip;
        let ray_eye = projection.calc_matrix().invert().unwrap() * ray_clip;
        let ray_eye = cgmath::Vector4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);

        //Step 4: 4d World Coordinates
        //vec3 ray_wor = (inverse(view_matrix) * ray_eye).xyz;
        let ray_wor_v4 = camera.calc_matrix().invert().unwrap() * ray_eye;
        let ray_wor : cgmath::Vector3<f32> = cgmath::Vector3::new(ray_wor_v4.x, ray_wor_v4.y, ray_wor_v4.z);

        // don't forget to normalise the vector at some point
        let ray_wor = ray_wor.normalize();

        //Use ray_wor to find right voxel


        cgmath::Vector3::new(1.0, 1.0, 1.0)
    }

}
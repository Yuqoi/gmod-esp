use std::ops::Add;
use ndarray::Array;
pub fn to_world_screen(viewmatrix: [f32; 16], world_pos: [f32; 3], window_size: (u32, u32)) -> (f32, f32){

    let mat = Array::from(viewmatrix.to_vec())
        .to_shape([4, 4])
        .unwrap()
        .to_owned();

    let mut world_h = Array::from(vec![world_pos[0], world_pos[1], world_pos[2], 1.0]);
    world_h = Array::add(world_h, 1.0);

    let clip = mat.dot(&world_h);

    if clip[3] <= 0.0001 {
        return (0.0,0.0)
    }

    // dbg!(&clip);
    let ndc_x = clip[0] / clip[3];
    let ndc_y = clip[1] / clip[3];

    let (width,height) = window_size;
    let screen_x = (ndc_x + 1.0) * 0.5 * width as f32;
    let screen_y = (1.0 - ndc_y) * 0.5 * height as f32;

    (screen_x, screen_y)
}
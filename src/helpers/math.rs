
pub fn to_world_screen(viewmatrix: [f32; 16], world_pos: [f32; 3], window_size: (u32, u32)) -> Option<(f32, f32)>{

    let [x, y, z] = world_pos;
    let w_in = [x, y, z, 1.0];

    let mut clip = [0.0f32; 4];
    for row in 0..4 {
        let base = row * 4;
        clip[row] = viewmatrix[base] * w_in[0]
            + viewmatrix[base + 1] * w_in[1]
            + viewmatrix[base + 2] * w_in[2]
            + viewmatrix[base + 3] * w_in[3];
    }

    if clip[3] <= 0.0001 {
        return None
    }

    let ndc_x = clip[0] / clip[3];
    let ndc_y = clip[1] / clip[3];

    let (width, height) = window_size;
    let screen_x = (ndc_x + 1.0) * 0.5 * width as f32;
    let screen_y = (1.0 - ndc_y) * 0.5 * height as f32;

    Some((screen_x, screen_y))
}
struct CameraUniform {
    view_proj: mat4x4<f32>,
    time: f32,
    zoom: f32,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // out.colour = model.colour;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.coord = model.coord;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // const small_grid_res: f32 = 8;
    // const small_grid_width: f32 = 1;
    // const big_grid_res: f32 = 80;
    // const big_grid_width: f32 = 5;
    // const big_grid_offset: f32 = (big_grid_width - small_grid_width) / 2.0;

    // let small_x =
    //     floor((small_grid_width / small_grid_res) + fract((in.clip_position.x) / small_grid_res));
    // let small_y =
    //     floor((small_grid_width / small_grid_res) + fract((in.clip_position.y) / small_grid_res));
    // let small_grid_factor = max(small_x, small_y);

    // let big_x = floor((big_grid_width / big_grid_res) + fract((in.clip_position.x) / big_grid_res));
    // let big_y = floor((big_grid_width / big_grid_res) + fract((in.clip_position.y) / big_grid_res));
    // let big_grid_factor = max(big_x, big_y);

    // let color = 1.0 - max(small_grid_factor, big_grid_factor);
    // return vec4<f32>(vec3(color), 1.0);

    const axis_width: f32 = 2;
    const grid_width: f32 = 1;
    let zoom: f32 = 100.0 * camera.zoom;

    let big_x = floor((grid_width / zoom) + fract(in.coord.x - grid_width / (zoom * 2.0)));
    let big_y = floor((grid_width / zoom) + fract(in.coord.y - grid_width / (zoom * 2.0)));
    let big_grid_factor = max(big_x, big_y) * 0.6;

    let small_x =
        floor((10.0 * grid_width / zoom) + fract(10.0 * (in.coord.x - grid_width / (zoom * 2.0))));
    let small_y =
        floor((10.0 * grid_width / zoom) + fract(10.0 * (in.coord.y - grid_width / (zoom * 2.0))));
    let small_grid_factor = max(small_x, small_y) * 0.3;

    let axis_factor = max(
            floor(1.0 - (abs(in.coord.x) - axis_width / (zoom * 2.0))),
            floor(1.0 - (abs(in.coord.y) - axis_width / (zoom * 2.0))),
        ) * 1.0;

    let color = 1.0 - max(axis_factor, max(big_grid_factor, small_grid_factor));
    return vec4<f32>(vec3(color), 1.0);
}

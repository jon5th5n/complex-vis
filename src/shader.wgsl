struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct Enviroment {
    range_start: vec2<f32>,
    range_end: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read> env: Enviroment;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let rxs = env.range_start.x;
    let rxe = env.range_end.x;
    let rys = env.range_start.y;
    let rye = env.range_end.y;

    let scale_x = 2.0 / (rxe - rxs);
    let trans_x = (-rxs - rxe) / (rxe - rxs);
    let scale_y = 2.0 / (rye - rys);
    let trans_y = (-rys - rye) / (rye - rys);

    let scale_trans_mat = mat3x3<f32>(vec3<f32>(scale_x, 0.0, 0.0), vec3<f32>(0.0, scale_y, 0.0), vec3<f32>(trans_x, trans_y, 1.0));

    let position = scale_trans_mat * vec3<f32>(model.position.xy, 1.0);

    out.clip_position = vec4<f32>(position, 1.0);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = in.color;
    return color;
}
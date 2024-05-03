@group(0) @binding(0)
var texture: texture_storage_2d<bgra8unorm, read_write>;

struct GraphingCanvasDescriptor {
    size: vec2<u32>,

    margin: vec2<u32>,

    range_start: vec2<f32>,
    range_end: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> canvas_desc: GraphingCanvasDescriptor;

struct GraphingFunction {
    color: vec4<f32>
}

@group(2) @binding(0)
var<uniform> graphing_function_vars: GraphingFunction;

const thickness: f32 = 5.0;
const subpixel: u32 = 2u;
const num_samples: u32 = (subpixel + 1u) * (subpixel + 1u);

@compute
@workgroup_size(1)
fn draw(@builtin(workgroup_id) id: vec3<u32>, @builtin(num_workgroups) size: vec3<u32>) {
    let half_pxl = (global_to_local(vec2<i32>(1, 1)) - global_to_local(vec2<i32>(0, 0))) / 2.0 * thickness;
    let quant = (global_to_local(vec2<i32>(1, 1)) - global_to_local(vec2<i32>(0, 0))) / f32(subpixel) * thickness;

    let pos = global_to_local(vec2<i32>(id.xy));

    var samples: array<f32, num_samples>;

    let start = pos - half_pxl;
    for (var i = 0u; i < num_samples; i++) {
        let x = i % (subpixel + 1);
        let y = i / (subpixel + 1u);

        let delta = quant * vec2<f32>(f32(x), f32(y));
        let current = start + delta;

        samples[i] = sign(func(current));
    }

    // samples[0] = sign(func(vec2<f32>(pos.x - half_pxl.x, pos.y - half_pxl.y)));
    // samples[1] = sign(func(vec2<f32>(pos.x + half_pxl.x, pos.y - half_pxl.y)));
    // samples[2] = sign(func(vec2<f32>(pos.x + half_pxl.x, pos.y + half_pxl.y)));
    // samples[3] = sign(func(vec2<f32>(pos.x - half_pxl.x, pos.y + half_pxl.y)));

    // var pos: u32 = 0u;
    // var neg: u32 = 0u;
    var sign_counter = 0.0;
    for (var i: u32 = 0u; i < num_samples; i++) {
        sign_counter += samples[i];
    }

    let val = 1.0 - smoothstep(0.0, 1.0, abs(sign_counter / f32(num_samples)));
    texture_add(id.xy, vec4<f32>(graphing_function_vars.color.xyz, graphing_function_vars.color.w * val));

    // let draw = !(sample1 == sample2 && sample1 == sample3 && sample1 == sample4);
    // if draw {
    //     texture_add(id.xy, graphing_function_vars.color);
    // };
}

fn colors_add(b: vec4<f32>, t: vec4<f32>) -> vec4<f32> {
    let bot = clamp(b, vec4<f32>(0.0, 0.0, 0.0, 0.0), vec4<f32>(1.0, 1.0, 1.0, 1.0));
    let top = clamp(t, vec4<f32>(0.0, 0.0, 0.0, 0.0), vec4<f32>(1.0, 1.0, 1.0, 1.0));

    let a = top.w + bot.w * (1.0 - top.w);
    let rgb = (top.xyz * top.w + bot.xyz * bot.w * (1.0 - top.w)) / a;
    return vec4<f32>(rgb, a);
}

fn texture_add(pos: vec2<u32>, color: vec4<f32>) {
    let c = colors_add(textureLoad(texture, pos), color);
    textureStore(texture, pos, c);
}

fn func(in: vec2<f32>) -> f32 {
    return pow(in.x, 2.0) - in.y;
    // return smoothstep(0.0, 1.0, in.x) * 5.0 - in.y;
}

fn global_to_local(global: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(
        (f32(global.x - i32(canvas_desc.margin.x)) / f32(drawing_size().x)) * range_len().x + canvas_desc.range_start.x,
        -((f32(global.y - i32(canvas_desc.margin.y)) / f32(drawing_size().y)) * range_len().y) + canvas_desc.range_end.y
    );
}

fn range_len() -> vec2<f32> {
    return abs(canvas_desc.range_end - canvas_desc.range_start);
}

fn drawing_size() -> vec2<u32> {
    return canvas_desc.size - 2u * canvas_desc.margin;
}
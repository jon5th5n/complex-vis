use crate::{GPUView, GPUViewFrame, ShaderDescriptor, Vertex};
use std::f32::consts::PI;
use std::{cell::RefCell, ops::Range, sync::Arc};

struct GPUCanvas2DShaderDescriptor {}

impl GPUCanvas2DShaderDescriptor {
    fn new() -> Self {
        Self {}
    }

    fn into_arc_ref_cell(self) -> Arc<RefCell<Self>> {
        Arc::new(RefCell::new(self))
    }
}

impl ShaderDescriptor for GPUCanvas2DShaderDescriptor {
    fn initialize(&mut self, device: &wgpu::Device) {
        return;
    }

    fn update_buffers(&self, queue: &wgpu::Queue) {
        return;
    }

    fn shader_source(&self) -> wgpu::ShaderSource {
        wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
    }

    fn bind_group_and_layout(
        &self,
        device: &wgpu::Device,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shader Descripot Bind Group Layout"),
            entries: &[],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shader Descriptor Bind Group"),
            layout: &bind_group_layout,
            entries: &[],
        });

        (bind_group, bind_group_layout)
    }
}

pub struct GPUCanvas2D<'a> {
    x_margin: f32,
    y_margin: f32,

    x_range: Range<f32>,
    y_range: Range<f32>,

    shader_descriptor: Arc<RefCell<GPUCanvas2DShaderDescriptor>>,
    view: Arc<RefCell<GPUView<'a>>>,
}

impl<'a> GPUCanvas2D<'a> {
    pub fn new(view_frame: GPUViewFrame) -> Self {
        let shader_descriptor = GPUCanvas2DShaderDescriptor::new().into_arc_ref_cell();

        Self {
            x_margin: 0.05,
            y_margin: 0.05,
            x_range: -1.0..1.0,
            y_range: -1.0..1.0,
            shader_descriptor: shader_descriptor.clone(),
            view: GPUView::new(view_frame, shader_descriptor).into_arc_ref_cell(),
        }
    }

    pub fn get_view(&self) -> Arc<RefCell<GPUView<'a>>> {
        self.view.clone()
    }

    pub fn set_clear_color(&mut self, clear_color: wgpu::Color) {
        self.view.as_ref().borrow_mut().set_clear_color(clear_color);
    }

    pub fn scale_range(&mut self, scale: f32) {
        let x_diff = self.x_range_len() * (scale - 1.0);
        let y_diff = self.y_range_len() * (scale - 1.0);

        self.x_range = (self.x_range.start - (x_diff * 0.5))..(self.x_range.end + (x_diff * 0.5));
        self.y_range = (self.y_range.start - (y_diff * 0.5))..(self.y_range.end + (y_diff * 0.5));
    }

    pub fn offset_range(&mut self, offset: (f32, f32)) {
        self.x_range = (self.x_range.start + offset.0)..(self.x_range.end + offset.0);
        self.y_range = (self.y_range.start + offset.1)..(self.y_range.end + offset.1);
    }

    fn x_range_len(&self) -> f32 {
        self.x_range.end - self.x_range.start
    }

    fn y_range_len(&self) -> f32 {
        self.y_range.end - self.y_range.start
    }

    fn global_to_screen(&self, global: (f32, f32)) -> (f32, f32) {
        let (gx, gy) = global;

        let lx = (2.0 * gx - self.x_range.start - self.x_range.end) / self.x_range_len();
        let ly = (2.0 * gy - self.y_range.start - self.y_range.end) / self.y_range_len();

        (lx, ly)
    }

    pub fn add_function(&mut self, f: fn(f32) -> f32) {
        let mut points = Vec::new();

        let sample_freq = 1000u32;

        let x_start = self.x_range.start;
        let x_len = self.x_range.end - x_start;

        let step = x_len / sample_freq as f32;

        for i in 0..=sample_freq {
            let gx = x_start + (step * i as f32);
            let gy = f(gx);

            let (lx, ly) = self.global_to_screen((gx, gy));

            points.push([lx, ly]);
        }

        let mut view = self.view.as_ref().borrow_mut();

        vertices_add_polyline(&mut view, points, 0.01, [1.0, 0.0, 0.0, 1.0]);
    }
}

fn vertices_add_polyline(view: &mut GPUView, points: Vec<[f32; 2]>, width: f32, color: [f32; 4]) {
    let mut last_point = None;
    for point in points {
        vertices_add_circle(view, point, width / 2.0, color, 8);

        if let Some(last_point) = last_point {
            vertices_add_line(view, last_point, point, width, color);
        }

        last_point = Some(point);
    }
}

fn vertices_add_line(
    view: &mut GPUView,
    end1: [f32; 2],
    end2: [f32; 2],
    width: f32,
    color: [f32; 4],
) {
    let normal = [end2[1] - end1[1], -(end2[0] - end1[0])];
    let normal_len = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();
    let normal_norm = [normal[0] / normal_len, normal[1] / normal_len];
    let normal_width = [normal_norm[0] * width, normal_norm[1] * width];

    let corner11 = [
        end1[0] + normal_width[0] / 2.0,
        end1[1] + normal_width[1] / 2.0,
    ];
    let corner12 = [
        end1[0] - normal_width[0] / 2.0,
        end1[1] - normal_width[1] / 2.0,
    ];
    let corner21 = [
        end2[0] + normal_width[0] / 2.0,
        end2[1] + normal_width[1] / 2.0,
    ];
    let corner22 = [
        end2[0] - normal_width[0] / 2.0,
        end2[1] - normal_width[1] / 2.0,
    ];

    view.append_render_vertices(&mut vec![
        Vertex {
            position: [corner11[0], corner11[1], 0.0],
            color,
        },
        Vertex {
            position: [corner12[0], corner12[1], 0.0],
            color,
        },
        Vertex {
            position: [corner21[0], corner21[1], 0.0],
            color,
        },
        Vertex {
            position: [corner12[0], corner12[1], 0.0],
            color,
        },
        Vertex {
            position: [corner21[0], corner21[1], 0.0],
            color,
        },
        Vertex {
            position: [corner22[0], corner22[1], 0.0],
            color,
        },
    ]);
}

fn vertices_add_circle(
    view: &mut GPUView,
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
    resolution: u8,
) {
    let resolution = 1usize << resolution.clamp(0, 8);
    let resolution = 256;

    let mut last_point: Option<[f32; 2]> = None;
    for i in 0..resolution {
        // let (sin, cos) = f32::sin_cos((i as f32 / resolution as f32) * 2.0 * PI);
        let sin_cos = CIRCLE_SIN_COS_LOOKUP[i];
        let sin = sin_cos[0];
        let cos = sin_cos[1];

        let x = center[0] + radius * cos;
        let y = center[1] + radius * sin;

        let point = [x, y];

        if let Some(last_point) = last_point {
            view.append_render_vertices(&mut vec![
                Vertex {
                    position: [last_point[0], last_point[1], 0.0],
                    color,
                },
                Vertex {
                    position: [center[0], center[1], 0.0],
                    color,
                },
                Vertex {
                    position: [point[0], point[1], 0.0],
                    color,
                },
            ]);
        }

        last_point = Some(point);
    }
}

static CIRCLE_SIN_COS_LOOKUP: [[f32; 2]; 256] = [
    [0.0, 1.0],
    [0.024541229, 0.9996988],
    [0.049067676, 0.99879545],
    [0.07356457, 0.99729043],
    [0.09801714, 0.9951847],
    [0.12241068, 0.99247956],
    [0.14673047, 0.9891765],
    [0.1709619, 0.98527765],
    [0.19509032, 0.98078525],
    [0.21910124, 0.9757021],
    [0.2429802, 0.97003126],
    [0.26671278, 0.96377605],
    [0.2902847, 0.95694035],
    [0.31368175, 0.94952816],
    [0.33688986, 0.94154406],
    [0.35989505, 0.9329928],
    [0.38268346, 0.9238795],
    [0.40524134, 0.9142097],
    [0.42755508, 0.9039893],
    [0.44961134, 0.8932243],
    [0.47139674, 0.88192123],
    [0.49289823, 0.87008697],
    [0.51410276, 0.8577286],
    [0.5349976, 0.8448536],
    [0.55557024, 0.8314696],
    [0.5758082, 0.8175848],
    [0.5956993, 0.8032075],
    [0.61523163, 0.7883464],
    [0.63439333, 0.77301043],
    [0.65317285, 0.7572088],
    [0.671559, 0.7409511],
    [0.68954057, 0.7242471],
    [0.70710677, 0.70710677],
    [0.7242471, 0.6895405],
    [0.7409512, 0.6715589],
    [0.7572089, 0.6531728],
    [0.77301043, 0.6343933],
    [0.7883464, 0.6152316],
    [0.8032075, 0.5956993],
    [0.8175848, 0.57580817],
    [0.83146966, 0.5555702],
    [0.8448536, 0.53499764],
    [0.85772866, 0.5141027],
    [0.87008697, 0.49289817],
    [0.8819213, 0.47139665],
    [0.8932243, 0.4496113],
    [0.9039893, 0.4275551],
    [0.9142098, 0.40524128],
    [0.9238795, 0.38268343],
    [0.9329928, 0.35989496],
    [0.94154406, 0.33688983],
    [0.9495282, 0.31368166],
    [0.95694035, 0.29028463],
    [0.96377605, 0.26671275],
    [0.97003126, 0.24298012],
    [0.9757021, 0.21910124],
    [0.9807853, 0.19509023],
    [0.98527765, 0.17096186],
    [0.9891765, 0.1467305],
    [0.99247956, 0.122410625],
    [0.9951847, 0.098017134],
    [0.99729043, 0.07356449],
    [0.99879545, 0.04906765],
    [0.9996988, 0.024541136],
    [1.0, -4.371139e-8],
    [0.9996988, -0.024541223],
    [0.99879545, -0.04906774],
    [0.99729043, -0.073564574],
    [0.9951847, -0.09801722],
    [0.9924795, -0.12241071],
    [0.9891765, -0.14673057],
    [0.98527765, -0.17096195],
    [0.98078525, -0.19509032],
    [0.9757021, -0.21910131],
    [0.97003126, -0.2429802],
    [0.96377605, -0.26671284],
    [0.9569403, -0.29028472],
    [0.94952816, -0.31368172],
    [0.94154406, -0.33688992],
    [0.9329928, -0.35989505],
    [0.9238795, -0.38268352],
    [0.9142097, -0.40524134],
    [0.9039893, -0.42755508],
    [0.8932243, -0.44961137],
    [0.88192123, -0.47139683],
    [0.870087, -0.49289817],
    [0.8577286, -0.51410276],
    [0.8448535, -0.5349977],
    [0.83146954, -0.55557036],
    [0.8175848, -0.57580817],
    [0.8032075, -0.59569937],
    [0.78834635, -0.6152317],
    [0.7730105, -0.6343933],
    [0.7572088, -0.65317285],
    [0.74095106, -0.67155904],
    [0.724247, -0.6895407],
    [0.70710677, -0.70710677],
    [0.6895405, -0.72424716],
    [0.67155886, -0.74095124],
    [0.65317285, -0.7572088],
    [0.6343933, -0.7730105],
    [0.6152315, -0.78834647],
    [0.59569913, -0.80320764],
    [0.57580817, -0.8175848],
    [0.5555702, -0.83146966],
    [0.53499746, -0.84485364],
    [0.51410276, -0.8577286],
    [0.49289814, -0.870087],
    [0.47139663, -0.88192135],
    [0.44961137, -0.8932243],
    [0.42755505, -0.9039893],
    [0.40524122, -0.9142098],
    [0.38268328, -0.9238796],
    [0.35989505, -0.9329928],
    [0.3368898, -0.9415441],
    [0.3136816, -0.9495282],
    [0.29028472, -0.95694035],
    [0.26671273, -0.96377605],
    [0.24298008, -0.97003126],
    [0.21910107, -0.97570217],
    [0.19509031, -0.9807853],
    [0.17096181, -0.98527765],
    [0.14673033, -0.9891765],
    [0.1224107, -0.9924795],
    [0.0980171, -0.9951847],
    [0.07356445, -0.9972905],
    [0.049067486, -0.99879545],
    [0.02454121, -0.9996988],
    [-8.742278e-8, -1.0],
    [-0.024541385, -0.9996988],
    [-0.04906766, -0.99879545],
    [-0.07356462, -0.99729043],
    [-0.09801727, -0.9951847],
    [-0.12241087, -0.9924795],
    [-0.1467305, -0.9891765],
    [-0.17096199, -0.98527765],
    [-0.19509049, -0.98078525],
    [-0.21910124, -0.9757021],
    [-0.24298024, -0.97003126],
    [-0.2667129, -0.96377605],
    [-0.29028487, -0.9569403],
    [-0.31368178, -0.94952816],
    [-0.33688995, -0.94154406],
    [-0.3598952, -0.93299276],
    [-0.38268343, -0.9238795],
    [-0.4052414, -0.9142097],
    [-0.42755523, -0.90398926],
    [-0.4496115, -0.8932242],
    [-0.47139677, -0.88192123],
    [-0.4928983, -0.8700869],
    [-0.5141029, -0.85772854],
    [-0.53499764, -0.8448536],
    [-0.5555703, -0.83146954],
    [-0.57580835, -0.8175847],
    [-0.5956993, -0.8032075],
    [-0.61523163, -0.7883464],
    [-0.6343934, -0.7730104],
    [-0.65317297, -0.7572087],
    [-0.671559, -0.7409511],
    [-0.6895406, -0.72424704],
    [-0.7071069, -0.70710665],
    [-0.7242471, -0.68954057],
    [-0.7409512, -0.6715589],
    [-0.75720876, -0.6531729],
    [-0.77301043, -0.63439333],
    [-0.78834647, -0.6152316],
    [-0.8032076, -0.5956992],
    [-0.81758493, -0.57580805],
    [-0.8314698, -0.55557],
    [-0.84485376, -0.53499734],
    [-0.85772854, -0.5141028],
    [-0.87008697, -0.4928982],
    [-0.8819213, -0.47139668],
    [-0.89322436, -0.44961122],
    [-0.9039894, -0.42755494],
    [-0.91420984, -0.40524107],
    [-0.9238797, -0.38268313],
    [-0.93299276, -0.3598951],
    [-0.94154406, -0.33688986],
    [-0.9495282, -0.3136817],
    [-0.95694035, -0.29028454],
    [-0.9637761, -0.26671258],
    [-0.9700313, -0.24297991],
    [-0.9757022, -0.2191009],
    [-0.98078525, -0.19509038],
    [-0.98527765, -0.17096189],
    [-0.9891765, -0.14673041],
    [-0.99247956, -0.122410536],
    [-0.9951847, -0.09801693],
    [-0.9972905, -0.07356428],
    [-0.99879545, -0.049067326],
    [-0.9996988, -0.024541287],
    [-1.0, 1.1924881e-8],
    [-0.9996988, 0.02454131],
    [-0.99879545, 0.049067825],
    [-0.99729043, 0.07356478],
    [-0.9951847, 0.09801743],
    [-0.9924795, 0.122411035],
    [-0.9891765, 0.14673042],
    [-0.98527765, 0.17096192],
    [-0.98078525, 0.19509041],
    [-0.9757021, 0.2191014],
    [-0.9700312, 0.2429804],
    [-0.963776, 0.26671305],
    [-0.95694023, 0.29028502],
    [-0.9495282, 0.3136817],
    [-0.94154406, 0.3368899],
    [-0.93299276, 0.35989514],
    [-0.92387944, 0.3826836],
    [-0.91420966, 0.40524155],
    [-0.90398914, 0.42755538],
    [-0.8932241, 0.44961166],
    [-0.8819213, 0.4713967],
    [-0.87008697, 0.49289823],
    [-0.85772854, 0.5141028],
    [-0.84485346, 0.53499776],
    [-0.8314695, 0.5555704],
    [-0.81758463, 0.57580847],
    [-0.8032076, 0.59569925],
    [-0.7883464, 0.6152316],
    [-0.77301043, 0.63439333],
    [-0.75720876, 0.6531729],
    [-0.740951, 0.6715591],
    [-0.7242469, 0.68954074],
    [-0.70710653, 0.707107],
    [-0.6895406, 0.72424704],
    [-0.671559, 0.7409511],
    [-0.6531728, 0.7572089],
    [-0.63439316, 0.77301055],
    [-0.61523145, 0.7883465],
    [-0.5956991, 0.8032077],
    [-0.5758079, 0.817585],
    [-0.5555703, 0.8314696],
    [-0.53499764, 0.8448536],
    [-0.5141027, 0.85772866],
    [-0.49289808, 0.8700871],
    [-0.47139654, 0.88192135],
    [-0.44961107, 0.8932244],
    [-0.4275548, 0.90398943],
    [-0.40524137, 0.9142097],
    [-0.38268343, 0.92387956],
    [-0.35989496, 0.9329928],
    [-0.3368897, 0.9415441],
    [-0.31368154, 0.9495283],
    [-0.2902844, 0.9569404],
    [-0.2667124, 0.9637762],
    [-0.24298023, 0.97003126],
    [-0.21910122, 0.9757021],
    [-0.19509023, 0.9807853],
    [-0.17096172, 0.98527765],
    [-0.14673024, 0.9891766],
    [-0.12241037, 0.99247956],
    [-0.09801677, 0.9951848],
    [-0.0735646, 0.99729043],
    [-0.04906764, 0.99879545],
    [-0.024541123, 0.9996988],
];

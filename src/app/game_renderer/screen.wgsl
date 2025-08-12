struct VertexIn {
    @builtin(vertex_index) idx: u32
};

struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Resolutions {
    src: vec2<f32>,
    dst: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> v_resolution: Resolutions;

@group(0) @binding(1)
var f_texture: texture_2d<f32>;

@group(0) @binding(2)
var f_sampler: sampler;


const vertexes = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

const uvs = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 0.0),
);

@vertex
fn vs_main(in: VertexIn) -> VertexOut {

    // Calculate aspect ratio scale
    let src_aspect = v_resolution.src.x / v_resolution.src.y;
    let dst_aspect = v_resolution.dst.x / v_resolution.dst.y;

    var scale = vec2<f32>(1.0, 1.0);

    if dst_aspect > src_aspect {
        // dst is wider, scale x down
        scale.x = src_aspect / dst_aspect;
    } else {
        // dst is taller, scale y down
        scale.y = dst_aspect / src_aspect;
    }

    let position_xy = vertexes[in.idx] * scale;

    var out: VertexOut;
    out.position = vec4<f32>(position_xy, 0.0, 1.0);
    out.uv = uvs[in.idx];
    return out;
}


@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(f_texture, f_sampler, in.uv);
}

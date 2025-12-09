// ---------------- Vertex Shader ----------------
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    // Fullscreen triangle
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    // Convert to UV (0~1)
    let uv = (pos[index] * vec2<f32>(0.5, -0.5)) + vec2<f32>(0.5, 0.5);

    var output: VertexOutput;
    output.position = vec4<f32>(pos[index], 0.0, 1.0);
    output.uv = uv;
    return output;
}


// ---------------- Fragment Shader ----------------
@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var samplr: sampler;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(tex, samplr, uv);
}

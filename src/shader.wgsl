



@group(0) @binding(0) var my_texture: texture_2d<f32>;


struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(idx) << 1u & 2i) * 2.0 - 1.0;
    let y = f32(i32(idx) & 2i) * -2.0 + 1.0;
    
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5)); // Map to 0.0-1.0
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coords = vec2<i32>(in.pos.xy);

    return textureLoad(my_texture, coords, 0);
}
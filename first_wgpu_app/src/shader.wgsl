@group(0) @binding(0) var<uniform> grid: vec2f;

@vertex
//pos is vertex attribute 0
fn vertex_main(
    @location(0) pos: vec2f,
    @builtin(instance_index) instance: u32
) -> @builtin(position) vec4f {

    let i = f32(instance);
    let cell = vec2f(i % grid.x, floor(i / grid.x));
    let cell_offset = (cell / grid) * 2;
    let grid_pos = (pos + 1) / grid - 1 + cell_offset;
    return vec4f(grid_pos, 0, 1);
}

@fragment
//output location is color attachment 0
fn fragment_main() -> @location(0) vec4f {
    return vec4f(1, 0, 0, 1); // (Red, Green, Blue, Alpha)
}
@group(0) @binding(0) var<uniform> grid: vec2f;

@vertex
//pos is vertex attribute 0
fn vertex_main(@location(0) pos: vec2f) 
    -> @builtin(position) vec4f {

    return vec4f(pos/grid, 0, 1);
}

@fragment
//output location is color attachment 0
fn fragment_main() -> @location(0) vec4f {
    return vec4f(1, 0, 0, 1); // (Red, Green, Blue, Alpha)
}
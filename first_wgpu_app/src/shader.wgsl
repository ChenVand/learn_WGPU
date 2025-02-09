@vertex
//pos ia vertwx attribute 0
fn vertex_main(@location(0) pos: vec2f) 
    -> @builtin(position) vec4f {

    return vec4f(pos, 0, 1);
}

@fragment
//output location is color attachment 0
fn fragment_main() -> @location(0) vec4f {
    return vec4f(1, 0, 0, 1); // (Red, Green, Blue, Alpha)
}
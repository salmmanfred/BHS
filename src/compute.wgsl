

struct Star{
    pos: vec3<f32>,
}



@group(0) @binding(0) var screen_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<storage, read> scene: array<Star>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let tex_size = textureDimensions(screen_tex);

    
    let color = vec4<f32>(0,0,0, 1.0);

    textureStore(screen_tex, id.xy, color);

    
    
    if (id.x == 1000u && id.y == 800u && id.z == 0u) {
        let color = vec4<f32>(1,1,1, 1.0);
        for (var k = 0u; k < arrayLength(&scene); k++) {
            let pos32 =  scene[k].pos;
            let pos = vec2<u32>(u32(pos32.x),u32(pos32.y));
            textureStore(screen_tex, pos, color);
        }
    }
    
}







//Stolen from https://github.com/SebLague/Ray-Tracing/blob/Episode01/Assets/Scripts/Shaders/RayTracing.shader
/* 
! might be usefull later
fn NextRandom(state: ptr<function, u32>)->u32
{
    *state = *state * 747796405 + 2891336453;
    var result = ((*state >> ((*state >> 28) + 4)) ^ *state) * 277803737;
    result = (result >> 22) ^ result;
    return result;
}

fn RandomValue(state: ptr<function, u32>)->f32
{   
    var a = f32(NextRandom(state)) / 4294967295.0;
    return a; // 2^32 - 1
}

// Random value in normal distribution (with mean=0 and sd=1)
fn RandomValueNormalDistribution(state: ptr<function, u32>)->f32
{
    // Thanks to https://stackoverflow.com/a/6178290
    var theta = 2 * 3.1415926 * RandomValue(state);
    var rho = sqrt(-2 * log(RandomValue(state)));
    return rho * cos(theta);
}

// Calculate a random direction
fn RandomDirection(state: ptr<function, u32>)->vec3<f32>
{
    // Thanks to https://math.stackexchange.com/a/1585996
    var x = RandomValueNormalDistribution(state);
    var y = RandomValueNormalDistribution(state);
    var z = RandomValueNormalDistribution(state);
    return normalize(vec3<f32>(x, y, z));
}*/
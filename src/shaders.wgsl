// src/shader.wgsl

// 1. ELJÁRÁS: LUT Generálás (33x33x33)

struct GpuColorSettings {
    setted: u32,
    gamma: f32,
    contrast: f32,
    brightness: f32,
    hue_shift: f32,
    saturation: f32,
    invert: u32,
    show_r: u32,
    show_g: u32,
    show_b: u32,
}

@group(0) @binding(0) var<uniform> colset: GpuColorSettings;
@group(0) @binding(1) var t_identity: texture_3d<f32>;
@group(0) @binding(2) var t_lut_out: texture_storage_3d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 4)
fn generate_lut(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= 33u || id.y >= 33u || id.z >= 33u) { return; }
    
    let raw = textureLoad(t_identity, vec3<i32>(id), 0).rgb;
    var color = raw; 
    if (colset.setted == 1u) {
        color = apply_color_math(color);
    }
    textureStore(t_lut_out, vec3<i32>(id), vec4<f32>(color, 1.0));
}

fn apply_color_math(in_color: vec3<f32>) -> vec3<f32> {
    var out = in_color;

    // 1. Invertálás
    if (colset.invert == 1u) { out = 1.0 - out; }

    // 2. HSV korrekciók
    var hsv = rgb_to_hsv(out);
    hsv.x = fract(hsv.x + colset.hue_shift / 360.0);
    if (colset.saturation > 0.0) {
        hsv.y = hsv.y + (1.0 - hsv.y) * colset.saturation;
    } else {
        hsv.y = hsv.y * (1.0 + colset.saturation);
    }
    out = hsv_to_rgb(hsv);

    // 3. Brightness, Contrast, Gamma
    let factor = (1.015 * (colset.contrast + 1.0)) / (1.015 - colset.contrast);
    out = factor * (out + colset.brightness - 0.5) + 0.5;
    out = pow(max(out, vec3(0.0)), vec3(1.0 / colset.gamma));

    // 4. channel mask
    let mask = vec3<f32>(f32(colset.show_r), f32(colset.show_g), f32(colset.show_b));
    return clamp(out * mask, vec3(0.0), vec3(1.0));
}


fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let v = max(c.r, max(c.g, c.b));
    let delta = v - min(c.r, min(c.g, c.b));
    var h = 0.0;
    var col = 0.0;
    if (v > 0.0) { col = delta / v; }
    if (delta > 0.0) {
        if (v == c.r) { h = (c.g - c.b) / delta + select(6.0, 0.0, c.g >= c.b); }
        else if (v == c.g) { h = (c.b - c.r) / delta + 2.0; }
        else { h = (c.r - c.g) / delta + 4.0; }
        h /= 6.0;
    }
    return vec3<f32>(h, col, v);
}

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
    return c.z * mix(k.xxx, clamp(p - k.xxx, vec3(0.0), vec3(1.0)), c.y);
}

// 2. ELJÁRÁS: Kép feldolgozása

struct FilterSettings {
    sharpen_radius: f32,   // > 0.2
    sharpen_amount: f32,   // 0.0 = kikapcsolva
    image_width: f32,
    image_height: f32,
}

// Bindingok az alkalmazáshoz
@group(1) @binding(0) var t_src: texture_2d<f32>;       // Eredeti kép
@group(1) @binding(1) var s_linear: sampler;            // Lineáris szűrő a LUT-hoz
@group(1) @binding(2) var t_lut: texture_3d<f32>;       // A már generált 3D LUT
@group(1) @binding(3) var<uniform> f: FilterSettings;
@group(1) @binding(4) var t_out: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(5) var<uniform> colset_apply: GpuColorSettings;

//@compute @workgroup_size(16, 16)
//fn apply_effects(@builtin(global_invocation_id) id: vec3<u32>) {
//    let dims_u32 = textureDimensions(t_src);
//    if (id.x >= dims_u32.x || id.y >= dims_u32.y) { return; }
//    let coords = vec2<i32>(id.xy);
//    let dims = vec2<i32>(dims_u32);
//    let center_color = textureLoad(t_src, coords, 0).rgb;
//    var processed = center_color;
//    var sum = vec3<f32>(0.0);
//    var count = 0.0;
//    let r = i32(f.sharpen_radius);
//    if (r > 0 && f.sharpen_amount != 0) {
//        for (var y: i32 = -r; y <= r; y++) {
//            for (var x: i32 = -r; x <= r; x++) {
//                let sample_coords = clamp(coords + vec2<i32>(x, y), vec2<i32>(0), dims - vec2<i32>(1));
//                sum += textureLoad(t_src, sample_coords, 0).rgb;
//                count += 1.0;
//            }
//        }
//        let average_color = sum / count;
//        let detail = center_color - average_color;
//        processed = center_color + detail * f.sharpen_amount;
//    }
//    let lut_size = 33.0;
//    let lut_coords = clamp(processed, vec3(0.0), vec3(1.0)) * ((lut_size - 1.0) / lut_size) + (0.5 / lut_size);
//    let corrected = textureSampleLevel(t_lut, s_linear, lut_coords, 0.0).rgb;
//    textureStore(t_out, coords, vec4<f32>(corrected, 1.0));
//}

@compute @workgroup_size(16, 16)
fn apply_effects(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims_u32 = textureDimensions(t_src);
    if (id.x >= dims_u32.x || id.y >= dims_u32.y) { return; }
    let coords = vec2<i32>(id.xy);
    let dims = vec2<i32>(dims_u32);
    let center_color = textureLoad(t_src, coords, 0).rgb;
    var processed = center_color;
	
    let r = i32(f.sharpen_radius*3.0+0.5) + 1;

    if (r > 0 && f.sharpen_radius >= 0.2 && f.sharpen_amount != 0) {
		let sigma = max(f.sharpen_radius / 2.0, 0.5);
		
		var weight = get_gaussian_weight(0.0, sigma); // center point
		var sample_coords = coords;
		var sum = textureLoad(t_src, sample_coords, 0).rgb * weight;
		var total_weight = weight;

        for (var i: i32 = 1; i <=r; i++) { // middle cross
            let dist = length(vec2<f32>(f32(i), 0.0));
            weight = get_gaussian_weight(dist, sigma);
            sample_coords = clamp(coords + vec2<i32>( i,  0), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            sample_coords = clamp(coords + vec2<i32>(-i,  0), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            sample_coords = clamp(coords + vec2<i32>( 0,  i), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            sample_coords = clamp(coords + vec2<i32>( 0, -i), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            total_weight += weight * 4.0;
        }
        for (var i: i32 = 1; i <=r; i++) { // diagonal
            let dist = length(vec2<f32>(f32(i), f32(i)));
            weight = get_gaussian_weight(dist, sigma);
            sample_coords = clamp(coords + vec2<i32>(i, i), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            sample_coords = clamp(coords + vec2<i32>(-i, i), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            sample_coords = clamp(coords + vec2<i32>(i, -i), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            sample_coords = clamp(coords + vec2<i32>(-i, -i), vec2<i32>(0), dims - vec2<i32>(1));
            sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
            total_weight += weight * 4.0;
        }
        for (var i: i32 =1; i <= r; i++) { // others
            for (var j: i32 = i+1; j <= r; j++) {
                let dist = length(vec2<f32>(f32(i), f32(j)));
                weight = get_gaussian_weight(dist, sigma);
                sample_coords = clamp(coords + vec2<i32>(i, j), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(-i, j), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(i, -j), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(-i, -j), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(j, i), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(-j, i), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(j, -i), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                sample_coords = clamp(coords + vec2<i32>(-j, -i), vec2<i32>(0), dims - vec2<i32>(1));
                sum += textureLoad(t_src, sample_coords, 0).rgb * weight;
                total_weight += weight * 8.0;
            }
        }
        let average_color = sum / total_weight;
        let detail = center_color - average_color;
        processed = center_color + detail * f.sharpen_amount;
    }
    let lut_size = 33.0;
    let lut_coords = clamp(processed, vec3(0.0), vec3(1.0)) * ((lut_size - 1.0) / lut_size) + (0.5 / lut_size);
    let corrected = textureSampleLevel(t_lut, s_linear, lut_coords, 0.0).rgb;
    textureStore(t_out, coords, vec4<f32>(corrected, 1.0));
}

fn get_gaussian_weight(dist: f32, sigma: f32) -> f32 {
    let s = 2.0 * sigma * sigma;
    return exp(-(dist * dist) / s);
}

// References:
// https://www.elopezr.com/temporal-aa-and-the-quest-for-the-holy-trail
// http://behindthepixels.io/assets/files/TemporalAA.pdf
// http://leiy.cc/publications/TAA/TAA_EG2020_Talk.pdf
// https://advances.realtimerendering.com/s2014/index.html#_HIGH-QUALITY_TEMPORAL_SUPERSAMPLING

// Controls how much to blend between the current and past samples
// Lower numbers = less of the current sample and more of the past sample = more smoothing
// Values chosen empirically

@group(0) @binding(0) var view_target: texture_2d<f32>;
@group(0) @binding(1) var history: texture_2d<f32>;
@group(0) @binding(2) var nearest_sampler: sampler;
@group(0) @binding(3) var linear_sampler: sampler;

struct Output {
    @location(0) view_target: vec4<f32>,
    @location(1) history: vec4<f32>,
};

fn sample_history(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(history, linear_sampler, uv).rgba;
}

@fragment
fn feedback(@location(0) uv: vec2<f32>) -> Output {

    let original_color = textureSample(view_target, nearest_sampler, uv);
    var current_color = original_color.rgba;
    var history_color = sample_history(uv);

    // lower opacity of history color
    history_color = history_color * 0.99;

    // history_color = mix(history_color, current_color, 0.00001);

    // history_color = mix(history_color, current_color);

    history_color = max(history_color, current_color);
    current_color = mix(history_color, current_color, 0.8);

    var out: Output;
    out.history = vec4(history_color);
    out.view_target = vec4(current_color);
    return out;
}

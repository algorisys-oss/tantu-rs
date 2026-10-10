// Shaders for tantu-render-wgpu. All colors are premultiplied, sRGB-encoded values; targets are
// non-sRGB formats, so blending happens on encoded values (as in the software renderer).
//
// Transforms are `[a, b, c, d, e, f]` (tantu_core::Affine) split into `t0 = (a, b, c, d)` and
// `t1 = (e, f)`: x' = a·x + c·y + e, y' = b·x + d·y + f.

struct Globals {
    viewport: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

fn to_clip(device: vec2<f32>) -> vec4<f32> {
    let n = device / globals.viewport * 2.0 - 1.0;
    return vec4<f32>(n.x, -n.y, 0.0, 1.0);
}

fn apply(t0: vec4<f32>, t1: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(t0.x * p.x + t0.z * p.y + t1.x, t0.y * p.x + t0.w * p.y + t1.y);
}

// The radius of the corner whose quadrant `q` (relative to the center, y down) is in.
// `radii` is (top-left, top-right, bottom-right, bottom-left).
fn corner_radius(q: vec2<f32>, radii: vec4<f32>) -> f32 {
    let left = select(radii.x, radii.w, q.y >= 0.0);
    let right = select(radii.y, radii.z, q.y >= 0.0);
    return select(left, right, q.x >= 0.0);
}

// Distance to the arc of a corner of radius `r`, for a point within the corner's square
// (`local` is measured from the rect's corner, pointing into the rect); a large negative value
// elsewhere, so the corner doesn't count.
fn sd_corner(local: vec2<f32>, r: f32) -> f32 {
    if (r > 0.0 && local.x < r && local.y < r) {
        return length(local - vec2<f32>(r)) - r;
    }
    return -3.0e38;
}

// Signed distance from `p` to a rounded rect `rect` = (left, top, right, bottom). Each corner
// owns the square of its own radius, not a quadrant around the center: a radius may exceed half
// a side when the opposite corner's radius is small (radii are already scaled to fit, the CSS
// rule), and its arc then reaches past the middle.
fn sd_rounded(p: vec2<f32>, rect: vec4<f32>, radii: vec4<f32>) -> f32 {
    let half = (rect.zw - rect.xy) * 0.5;
    let q = abs(p - (rect.xy + rect.zw) * 0.5) - half;
    var d = min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0)));
    d = max(d, sd_corner(p - rect.xy, radii.x));
    d = max(d, sd_corner(vec2<f32>(rect.z - p.x, p.y - rect.y), radii.y));
    d = max(d, sd_corner(rect.zw - p, radii.z));
    d = max(d, sd_corner(vec2<f32>(p.x - rect.x, rect.w - p.y), radii.w));
    return d;
}

// Anti-aliased coverage of the region where `d` < 0, with a one-device-pixel ramp. The pixel
// size in local units comes from the screen-space derivatives of the local position `p`, which
// is affine in screen space, so they are exact (the distance's own gradient jumps at corners).
fn coverage(d: f32, p: vec2<f32>) -> f32 {
    let w = max(0.5 * (length(dpdx(p)) + length(dpdy(p))), 1e-12);
    return clamp(0.5 - d / w, 0.0, 1.0);
}

// ---- Gaussian-blurred rounded rect (Evan Wallace's closed form) ------------------------------

fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (2.5066282746 * sigma);
}

fn erf2(x: vec2<f32>) -> vec2<f32> {
    let s = sign(x);
    let a = abs(x);
    var r = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    r = r * r;
    return s - s / (r * r);
}

fn shadow_x(x: f32, y: f32, sigma: f32, corner: f32, half: vec2<f32>) -> f32 {
    let delta = min(half.y - corner - abs(y), 0.0);
    let curved = half.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    let integral = 0.5 + 0.5 * erf2((x + vec2<f32>(-curved, curved)) * (sqrt(0.5) / sigma));
    return integral.y - integral.x;
}

fn shadow_value(p: vec2<f32>, rect: vec4<f32>, radii: vec4<f32>, sigma: f32) -> f32 {
    let half = (rect.zw - rect.xy) * 0.5;
    let q = p - (rect.xy + rect.zw) * 0.5;
    let corner = min(corner_radius(q, radii), min(half.x, half.y));
    let low = q.y - half.y;
    let high = q.y + half.y;
    let start = clamp(-3.0 * sigma, low, high);
    let end = clamp(3.0 * sigma, low, high);
    let step = (end - start) / 8.0;
    var y = start + step * 0.5;
    var value = 0.0;
    for (var i = 0; i < 8; i++) {
        value += shadow_x(q.x, q.y - y, sigma, corner, half) * gaussian(y, sigma) * step;
        y += step;
    }
    return clamp(value, 0.0, 1.0);
}

// ---- Group 1: a target-sized texture read by pixel position -----------------------------------
// The clip mask (R8) for shapes and images, or the layer texture for compositing.

@group(1) @binding(0) var pixel_tex: texture_2d<f32>;

fn mask_at(pos: vec4<f32>) -> f32 {
    return textureLoad(pixel_tex, vec2<i32>(pos.xy), 0).r;
}

fn quad_corner(vi: u32) -> vec2<f32> {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    return corners[vi];
}

// ---- Shapes: fill (kind 0), stroke (kind 1), blurred shadow (kind 2) ------------------------

struct ShapeIn {
    @location(0) t0: vec4<f32>,
    @location(1) t1: vec2<f32>,
    @location(2) rect: vec4<f32>,
    @location(3) radii: vec4<f32>,
    @location(4) color: vec4<f32>,
    // (kind, stroke width or blur sigma, quad margin, unused), lengths in local units.
    @location(5) params: vec4<f32>,
};

struct ShapeOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) rect: vec4<f32>,
    @location(2) @interpolate(flat) radii: vec4<f32>,
    @location(3) @interpolate(flat) color: vec4<f32>,
    @location(4) @interpolate(flat) params: vec4<f32>,
};

@vertex
fn vs_shape(@builtin(vertex_index) vi: u32, inst: ShapeIn) -> ShapeOut {
    let m = vec2<f32>(inst.params.z);
    let local = mix(inst.rect.xy - m, inst.rect.zw + m, quad_corner(vi));
    var out: ShapeOut;
    out.pos = to_clip(apply(inst.t0, inst.t1, local));
    out.local = local;
    out.rect = inst.rect;
    out.radii = inst.radii;
    out.color = inst.color;
    out.params = inst.params;
    return out;
}

@fragment
fn fs_shape(in: ShapeOut) -> @location(0) vec4<f32> {
    let w = in.params.y;
    let outer = coverage(sd_rounded(in.local, in.rect, in.radii), in.local);
    let inner_rect = in.rect + vec4<f32>(w, w, -w, -w);
    let inner_radii = max(in.radii - vec4<f32>(w), vec4<f32>(0.0));
    let inner = coverage(sd_rounded(in.local, inner_rect, inner_radii), in.local);
    let blurred = shadow_value(in.local, in.rect, in.radii, max(w, 1e-3));
    var cov = outer;
    if (in.params.x > 1.5) {
        cov = blurred;
    } else if (in.params.x > 0.5) {
        cov = outer * (1.0 - inner);
    }
    return in.color * (cov * mask_at(in.pos));
}

// ---- Images ----------------------------------------------------------------------------------

struct ImageIn {
    @location(0) t0: vec4<f32>,
    @location(1) t1: vec2<f32>,
    @location(2) dest: vec4<f32>,
    // (u0, v0, u1, v1): the source rect in normalized texture coordinates.
    @location(3) uv: vec4<f32>,
    // (opacity, quad margin)
    @location(4) extra: vec2<f32>,
};

struct ImageOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) dest: vec4<f32>,
    @location(2) @interpolate(flat) uv: vec4<f32>,
    @location(3) @interpolate(flat) extra: vec2<f32>,
};

@group(2) @binding(0) var image_tex: texture_2d<f32>;
@group(2) @binding(1) var image_sampler: sampler;

@vertex
fn vs_image(@builtin(vertex_index) vi: u32, inst: ImageIn) -> ImageOut {
    let m = vec2<f32>(inst.extra.y);
    let local = mix(inst.dest.xy - m, inst.dest.zw + m, quad_corner(vi));
    var out: ImageOut;
    out.pos = to_clip(apply(inst.t0, inst.t1, local));
    out.local = local;
    out.dest = inst.dest;
    out.uv = inst.uv;
    out.extra = inst.extra;
    return out;
}

@fragment
fn fs_image(in: ImageOut) -> @location(0) vec4<f32> {
    let t = (in.local - in.dest.xy) / (in.dest.zw - in.dest.xy);
    let uv = mix(in.uv.xy, in.uv.zw, t);
    let color = textureSample(image_tex, image_sampler, uv);
    let cov = coverage(sd_rounded(in.local, in.dest, vec4<f32>(0.0)), in.local);
    return color * (in.extra.x * cov * mask_at(in.pos));
}

// ---- Full-screen passes: clip masks, layer overlay and composite -----------------------------

struct FullIn {
    @location(0) p0: vec4<f32>,
    @location(1) p1: vec4<f32>,
    @location(2) p2: vec4<f32>,
    @location(3) p3: vec4<f32>,
};

struct FullOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) p0: vec4<f32>,
    @location(1) @interpolate(flat) p1: vec4<f32>,
    @location(2) @interpolate(flat) p2: vec4<f32>,
    @location(3) @interpolate(flat) p3: vec4<f32>,
};

@vertex
fn vs_full(@builtin(vertex_index) vi: u32, inst: FullIn) -> FullOut {
    var corners = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var out: FullOut;
    out.pos = vec4<f32>(corners[vi], 0.0, 1.0);
    out.p0 = inst.p0;
    out.p1 = inst.p1;
    out.p2 = inst.p2;
    out.p3 = inst.p3;
    return out;
}

// Clip coverage: p0/p1.xy = device-to-local transform, p2 = rect, p3 = radii. Blended as
// dst × src, so the mask becomes the parent mask times this clip.
@fragment
fn fs_clip(in: FullOut) -> @location(0) vec4<f32> {
    let local = apply(in.p0, in.p1.xy, in.pos.xy);
    let cov = coverage(sd_rounded(local, in.p2, in.p3), local);
    return vec4<f32>(cov, cov, cov, cov);
}

// Overlay color (p0, premultiplied), blended source-atop onto a layer.
@fragment
fn fs_overlay(in: FullOut) -> @location(0) vec4<f32> {
    return in.p0;
}

// A layer texture (group 1) times its opacity (p0.x), blended source-over.
@fragment
fn fs_composite(in: FullOut) -> @location(0) vec4<f32> {
    return textureLoad(pixel_tex, vec2<i32>(in.pos.xy), 0) * in.p0.x;
}

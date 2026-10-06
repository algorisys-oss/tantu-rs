//! Gaussian blur for box shadows, approximated by three box blurs per axis.

use tiny_skia::{FillRule, Mask, Paint, Path, Pixmap, Transform};

/// Buffers reused between shadows and frames.
#[derive(Default)]
pub(crate) struct Scratch {
    /// The shape, drawn with a margin around the target so the blur sees content just outside it.
    padded: Option<Mask>,
    /// The blurred shape cropped to the target and multiplied by the clip.
    cropped: Option<Mask>,
    line: Vec<u8>,
    out: Vec<u8>,
}

/// Returns `slot`'s mask, replacing it when it doesn't have the size `w × h`.
fn mask_of_size(slot: &mut Option<Mask>, w: u32, h: u32) -> Option<&mut Mask> {
    if slot
        .as_ref()
        .is_none_or(|m| (m.width(), m.height()) != (w, h))
    {
        *slot = Some(Mask::new(w, h)?);
    }
    slot.as_mut()
}

/// Fills `path` (in device pixels) blurred with a Gaussian of `sigma` device pixels, with
/// `paint`, limited by `clip`.
pub(crate) fn fill_blurred(
    target: &mut Pixmap,
    clip: Option<&Mask>,
    path: &Path,
    sigma: f32,
    paint: &Paint<'_>,
    scratch: &mut Scratch,
) {
    let (w, h) = (target.width(), target.height());
    // Content further than 3 sigma away doesn't show; beyond the target's size it is capped
    // so huge blurs stay bounded in memory and time.
    let margin = if sigma.is_finite() {
        (3.0 * sigma).ceil().min(w.max(h) as f32) as u32
    } else {
        w.max(h)
    };
    let (Some(pw), Some(ph)) = (
        w.checked_add(margin.saturating_mul(2)),
        h.checked_add(margin.saturating_mul(2)),
    ) else {
        return;
    };
    let Some(padded) = mask_of_size(&mut scratch.padded, pw, ph) else {
        return;
    };
    padded.clear();
    padded.fill_path(
        path,
        FillRule::Winding,
        true,
        Transform::from_translate(margin as f32, margin as f32),
    );
    blur_mask(padded, sigma, &mut scratch.line, &mut scratch.out);

    let Some(cropped) = mask_of_size(&mut scratch.cropped, w, h) else {
        return;
    };
    let (w, h, pw, m) = (w as usize, h as usize, pw as usize, margin as usize);
    let src = padded.data();
    let dst = cropped.data_mut();
    for y in 0..h {
        let from = (y + m) * pw + m;
        dst[y * w..(y + 1) * w].copy_from_slice(&src[from..from + w]);
    }
    if let Some(clip) = clip {
        for (d, c) in dst.iter_mut().zip(clip.data()) {
            *d = mul_u8(*d, *c);
        }
    }
    if let Some(full) = tiny_skia::Rect::from_xywh(0.0, 0.0, w as f32, h as f32) {
        target.fill_rect(full, paint, Transform::identity(), Some(cropped));
    }
}

/// `a · b / 255`, rounded.
fn mul_u8(a: u8, b: u8) -> u8 {
    ((a as u32 * b as u32 + 127) / 255) as u8
}

/// Widths of the three box blurs approximating a Gaussian of `sigma` (W. Jarosz / P. Kovesi).
fn box_sizes(sigma: f32, cap: usize) -> [usize; 3] {
    let n = 3.0f64;
    let s = sigma as f64;
    let ideal = (12.0 * s * s / n + 1.0).sqrt();
    let mut lower = ideal.floor().max(1.0);
    if lower % 2.0 == 0.0 {
        lower -= 1.0;
    }
    let upper = lower + 2.0;
    let m = ((12.0 * s * s - n * lower * lower - 4.0 * n * lower - 3.0 * n) / (-4.0 * lower - 4.0))
        .round();
    let cap = (2 * cap + 1) as f64;
    let size = |i: f64| if i < m { lower } else { upper }.min(cap) as usize;
    [size(0.0), size(1.0), size(2.0)]
}

/// Blurs the mask in place: three box blurs along rows, then along columns.
fn blur_mask(mask: &mut Mask, sigma: f32, line: &mut Vec<u8>, out: &mut Vec<u8>) {
    let (w, h) = (mask.width() as usize, mask.height() as usize);
    let sizes = box_sizes(sigma, w.max(h));
    let data = mask.data_mut();
    for row in data.chunks_exact_mut(w) {
        for &size in &sizes {
            box_blur_line(row, size / 2, out);
        }
    }
    line.resize(h, 0);
    for x in 0..w {
        for y in 0..h {
            line[y] = data[y * w + x];
        }
        for &size in &sizes {
            box_blur_line(line, size / 2, out);
        }
        for y in 0..h {
            data[y * w + x] = line[y];
        }
    }
}

/// Replaces each value with the average over `[i - radius, i + radius]`, treating values
/// outside the line as 0.
fn box_blur_line(values: &mut [u8], radius: usize, out: &mut Vec<u8>) {
    let n = values.len();
    if radius == 0 || n == 0 {
        return;
    }
    let width = (2 * radius + 1) as u64;
    out.clear();
    out.resize(n, 0);
    // Sum of values[i - radius ..= i + radius], clamped to the line.
    let mut sum: u64 = values[..(radius + 1).min(n)]
        .iter()
        .map(|&v| v as u64)
        .sum();
    for i in 0..n {
        out[i] = ((sum + width / 2) / width) as u8;
        if let Some(&v) = values.get(i + radius + 1) {
            sum += v as u64;
        }
        if i >= radius {
            sum -= values[i - radius] as u64;
        }
    }
    values.copy_from_slice(out);
}

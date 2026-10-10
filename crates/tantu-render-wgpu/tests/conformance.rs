//! The Phase 1 milestone: wgpu against the reference goldens (spec
//! `docs/specs/render-conformance/conformance.md`, RENDER-CONF-12). Skips without a GPU adapter
//! unless `TANTU_REQUIRE_GPU=1`. A failed rendering is written next to its golden as
//! `<name>.wgpu.actual.png` (gitignored).

mod common;

use std::path::Path;

use common::gpu;
use tantu_render_conformance::{MatchTolerance, reference_scenes};
use tantu_scene::{ImageData, Renderer};

fn write_png(path: &Path, image: &ImageData) {
    let file = std::fs::File::create(path).expect("create PNG file");
    let mut encoder =
        png::Encoder::new(std::io::BufWriter::new(file), image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("PNG header");
    writer.write_image_data(image.pixels()).expect("PNG data");
}

#[test]
fn render_conf_12_wgpu_matches_the_goldens() {
    let mut failures = Vec::new();
    for reference in reference_scenes() {
        let (w, h) = reference.target_size();
        let Some(mut renderer) = gpu(w, h) else {
            return;
        };
        renderer.resize(w, h, reference.scale_factor());
        let (scene, resources) = reference.record();
        let report = renderer
            .render(&scene, &resources)
            .expect("offscreen target");
        assert!(report.is_clean(), "{}: {report:?}", reference.name());
        let image = renderer.snapshot().expect("offscreen, non-empty");
        let result = reference
            .check(&image, &MatchTolerance::CROSS_BACKEND)
            .expect("golden has the target size");
        if !result.passed {
            if failures.is_empty() {
                failures.push(format!("adapter: {:?}", renderer.adapter_info()));
            }
            let path = reference.golden_path().with_extension("wgpu.actual.png");
            write_png(&path, &image);
            failures.push(format!(
                "{}: {result:?}; rendering written to {}",
                reference.name(),
                path.display()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

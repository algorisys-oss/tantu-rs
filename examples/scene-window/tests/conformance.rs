//! The demo frame rendered by wgpu and by the software renderer (spec
//! `docs/specs/render-conformance/conformance.md`, RENDER-CONF-13). Skips without a GPU adapter
//! unless `TANTU_REQUIRE_GPU=1`.

use scene_window::{demo_image, demo_scene};
use tantu_core::Size;
use tantu_render_conformance::{MatchTolerance, match_images};
use tantu_render_soft::SoftRenderer;
use tantu_render_wgpu::{CreateError, WgpuRenderer};
use tantu_scene::{Renderer, Resources, Scene};

#[test]
#[allow(clippy::print_stderr)] // Say why the test did nothing.
fn render_conf_13_demo_frame_matches_across_backends() {
    let (w, h) = (900, 600);
    let mut resources = Resources::new();
    let image = resources.add_image(demo_image());
    for scale in [1.0f32, 2.0] {
        let mut scene = Scene::new();
        demo_scene(
            &mut scene,
            Size::new(w as f32 / scale, h as f32 / scale),
            0.5,
            image,
        );

        let mut soft = SoftRenderer::new(w, h);
        soft.resize(w, h, scale);
        let report = soft.render(&scene, &resources).expect("in-memory target");
        assert!(report.is_clean(), "soft at scale {scale}: {report:?}");
        let reference = soft.snapshot().expect("non-empty");

        let mut gpu = match WgpuRenderer::new_offscreen(w, h) {
            Ok(renderer) => renderer,
            Err(CreateError::NoAdapter)
                if std::env::var_os("TANTU_REQUIRE_GPU").is_none_or(|v| v != "1") =>
            {
                eprintln!("skipped: no GPU adapter (set TANTU_REQUIRE_GPU=1 to fail instead)");
                return;
            }
            Err(e) => panic!("creating a wgpu renderer failed: {e}"),
        };
        gpu.resize(w, h, scale);
        let report = gpu.render(&scene, &resources).expect("offscreen target");
        assert!(report.is_clean(), "wgpu at scale {scale}: {report:?}");
        let actual = gpu.snapshot().expect("offscreen, non-empty");

        let result =
            match_images(&reference, &actual, &MatchTolerance::CROSS_BACKEND).expect("same size");
        assert!(
            result.passed,
            "scale {scale}: {result:?}; adapter {:?}",
            gpu.adapter_info()
        );
    }
}

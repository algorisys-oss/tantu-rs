//! Tests for `docs/specs/scene/renderer.md`, rules SCENE-RES-NN (image/font data, `Resources`).

use std::collections::HashSet;
use std::sync::Arc;

use tantu_scene::{FontData, ImageData, ResourceError, Resources};

fn image(w: u32, h: u32) -> ImageData {
    ImageData::rgba8(w, h, vec![0u8; (w * h * 4) as usize]).expect("valid size and length")
}

fn font() -> FontData {
    FontData::new(vec![1u8, 2, 3], 0).expect("non-empty")
}

// ---- Image and font data -----------------------------------------------------------------

#[test]
fn scene_res_01_valid_image_data() {
    let pixels: Vec<u8> = (0..24).collect();
    let data = ImageData::rgba8(3, 2, pixels.clone()).unwrap();
    assert_eq!(data.width(), 3);
    assert_eq!(data.height(), 2);
    assert_eq!(data.pixels(), &pixels[..]);

    let shared: Arc<[u8]> = Arc::from(&[9u8, 9, 9, 9][..]);
    assert_eq!(
        ImageData::rgba8(1, 1, shared).unwrap().pixels(),
        &[9, 9, 9, 9]
    );
}

#[test]
fn scene_res_02_invalid_image_size() {
    assert_eq!(
        ImageData::rgba8(0, 2, Vec::new()),
        Err(ResourceError::InvalidImageSize {
            width: 0,
            height: 2
        })
    );
    assert_eq!(
        ImageData::rgba8(3, 0, vec![0u8; 12]),
        Err(ResourceError::InvalidImageSize {
            width: 3,
            height: 0
        })
    );
    // Checked before the length: an overflowing size with a short buffer is a size error.
    if usize::BITS <= 64 {
        assert_eq!(
            ImageData::rgba8(u32::MAX, u32::MAX, Vec::new()),
            Err(ResourceError::InvalidImageSize {
                width: u32::MAX,
                height: u32::MAX
            })
        );
    }
}

#[test]
fn scene_res_03_wrong_pixel_buffer_length() {
    assert_eq!(
        ImageData::rgba8(2, 2, vec![0u8; 15]),
        Err(ResourceError::PixelDataLength {
            expected: 16,
            actual: 15
        })
    );
    assert_eq!(
        ImageData::rgba8(2, 2, vec![0u8; 17]),
        Err(ResourceError::PixelDataLength {
            expected: 16,
            actual: 17
        })
    );
    assert_eq!(
        ImageData::rgba8(1, 1, Vec::new()),
        Err(ResourceError::PixelDataLength {
            expected: 4,
            actual: 0
        })
    );
}

#[test]
fn scene_res_04_font_data() {
    assert_eq!(
        FontData::new(Vec::new(), 0),
        Err(ResourceError::EmptyFontData)
    );
    // Not parsed, index not checked.
    let data = FontData::new(b"not a font".to_vec(), 7).unwrap();
    assert_eq!(data.bytes(), b"not a font");
    assert_eq!(data.index(), 7);
}

#[test]
fn scene_res_05_clones_share_buffers_and_debug_hides_them() {
    let img = ImageData::rgba8(1, 1, vec![201u8, 202, 203, 204]).unwrap();
    let img2 = img.clone();
    assert_eq!(img, img2);
    assert!(std::ptr::eq(img.pixels().as_ptr(), img2.pixels().as_ptr()));
    let debug = format!("{img:?}");
    assert!(!debug.contains("201") && !debug.contains("204"), "{debug}");

    let f = FontData::new(vec![171u8, 172, 173], 0).unwrap();
    let f2 = f.clone();
    assert!(std::ptr::eq(f.bytes().as_ptr(), f2.bytes().as_ptr()));
    let debug = format!("{f:?}");
    assert!(!debug.contains("171") && !debug.contains("173"), "{debug}");
}

// ---- Resources ---------------------------------------------------------------------------

#[test]
fn scene_res_06_handles_are_unique_and_never_reused() {
    let mut a = Resources::new();
    let mut b = Resources::new();
    let mut images = HashSet::new();
    let mut fonts = HashSet::new();
    for _ in 0..50 {
        assert!(images.insert(a.add_image(image(1, 1))));
        assert!(images.insert(b.add_image(image(1, 1))));
        assert!(fonts.insert(a.add_font(font())));
        assert!(fonts.insert(b.add_font(font())));
    }
    let removed: Vec<_> = images.iter().copied().take(10).collect();
    for id in removed {
        a.remove_image(id);
        b.remove_image(id);
    }
    for _ in 0..10 {
        assert!(images.insert(a.add_image(image(1, 1))));
        assert!(fonts.insert(a.add_font(font())));
    }
    // Handles issued on other threads are unique too.
    let other: Vec<_> = std::thread::spawn(|| {
        let mut r = Resources::new();
        (0..50).map(|_| r.add_image(image(1, 1))).collect()
    })
    .join()
    .unwrap();
    for id in other {
        assert!(images.insert(id));
    }
}

#[test]
fn scene_res_07_lookup_until_removed() {
    let mut res = Resources::new();
    let img = image(2, 3);
    let f = font();
    let image_id = res.add_image(img.clone());
    let font_id = res.add_font(f.clone());
    assert_eq!(res.image(image_id), Some(&img));
    assert_eq!(res.font(font_id), Some(&f));

    res.remove_image(image_id);
    res.remove_font(font_id);
    assert_eq!(res.image(image_id), None);
    assert_eq!(res.font(font_id), None);

    let mut other = Resources::new();
    let foreign_image = other.add_image(image(1, 1));
    let foreign_font = other.add_font(font());
    assert_eq!(res.image(foreign_image), None);
    assert_eq!(res.font(foreign_font), None);
}

#[test]
fn scene_res_08_remove_returns_data_or_none() {
    let mut res = Resources::new();
    let img = image(1, 2);
    let f = font();
    let image_id = res.add_image(img.clone());
    let font_id = res.add_font(f.clone());
    let keep = res.add_image(image(4, 4));

    assert_eq!(res.remove_image(image_id), Some(img));
    assert_eq!(res.remove_image(image_id), None);
    assert_eq!(res.remove_font(font_id), Some(f));
    assert_eq!(res.remove_font(font_id), None);
    assert_eq!(res.image(keep).map(ImageData::width), Some(4));
}

#[test]
fn scene_res_09_revision() {
    let mut res = Resources::new();
    assert_eq!(res.revision(), 0);
    let i = res.add_image(image(1, 1));
    assert_eq!(res.revision(), 1);
    let f = res.add_font(font());
    assert_eq!(res.revision(), 2);

    let _ = res.image(i);
    let _ = res.font(f);
    assert_eq!(res.revision(), 2);

    res.remove_image(i);
    assert_eq!(res.revision(), 3);
    res.remove_image(i);
    assert_eq!(res.revision(), 3);
    res.remove_font(f);
    assert_eq!(res.revision(), 4);
    res.remove_font(f);
    assert_eq!(res.revision(), 4);
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn scene_res_10_clones_are_independent() {
    let mut a = Resources::new();
    let i = a.add_image(image(1, 1));
    let b = a.clone();
    assert_eq!(b.revision(), a.revision());
    assert_eq!(b.image(i), a.image(i));

    let f = a.add_font(font());
    a.remove_image(i);
    assert!(b.image(i).is_some());
    assert!(b.font(f).is_none());
    assert_eq!(b.revision(), 1);
    assert_eq!(a.revision(), 3);

    assert_send_sync::<Resources>();
    assert_send_sync::<ImageData>();
    assert_send_sync::<FontData>();
}

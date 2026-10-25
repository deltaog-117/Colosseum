use render::resolution::{RenderResolution, Viewport, ZeroResolution};

fn fitted(width: u32, height: u32) -> Viewport {
    let Some(view) = RenderResolution::DEFAULT.fit(width, height) else {
        panic!("a {width}x{height} window has a viewport");
    };
    view
}

#[test]
fn zero_sides_are_rejected() {
    assert_eq!(RenderResolution::try_new(0, 240), Err(ZeroResolution));
    assert_eq!(RenderResolution::try_new(320, 0), Err(ZeroResolution));
}

#[test]
fn default_is_320_by_240() {
    let res = RenderResolution::DEFAULT;
    assert_eq!((res.width(), res.height()), (320, 240));
    assert!((res.aspect() - 4.0 / 3.0).abs() < 1e-6);
}

#[test]
fn a_zero_window_has_no_viewport() {
    assert!(RenderResolution::DEFAULT.fit(0, 600).is_none());
    assert!(RenderResolution::DEFAULT.fit(800, 0).is_none());
}

#[test]
fn an_exact_multiple_fills_the_window() {
    let view = fitted(960, 720);
    assert_eq!(
        (view.x(), view.y(), view.width(), view.height()),
        (0, 0, 960, 720)
    );
}

#[test]
fn a_wide_window_gets_side_bars_at_a_whole_scale() {
    let view = fitted(1920, 1080);
    // 1080 / 240 = 4, so 1280x960 centered.
    assert_eq!(
        (view.x(), view.y(), view.width(), view.height()),
        (320, 60, 1280, 960)
    );
}

#[test]
fn a_small_window_scales_down_to_fit() {
    let view = fitted(160, 200);
    assert_eq!((view.width(), view.height()), (160, 120));
    assert_eq!((view.x(), view.y()), (0, 40));
}

use proptest::prelude::*;
use render::resolution::{RenderResolution, Viewport};

fn fitted(res: RenderResolution, width: u32, height: u32) -> Viewport {
    let Some(view) = res.fit(width, height) else {
        panic!("a {width}x{height} window has a viewport");
    };
    view
}

fn resolution() -> impl Strategy<Value = RenderResolution> {
    // Ranges start at 1, so the zero check cannot fail.
    (1u32..2000, 1u32..2000).prop_filter_map("non-zero sides", |(w, h)| {
        RenderResolution::try_new(w, h).ok()
    })
}

proptest! {
    #[test]
    fn the_viewport_is_non_empty_and_inside_the_window(
        res in resolution(),
        // Up to u32::MAX: the fit arithmetic must not overflow.
        ww in 1u32.., wh in 1u32..,
    ) {
        let view = fitted(res, ww, wh);
        prop_assert!(view.width() >= 1 && view.height() >= 1);
        prop_assert!(u64::from(view.x()) + u64::from(view.width()) <= u64::from(ww));
        prop_assert!(u64::from(view.y()) + u64::from(view.height()) <= u64::from(wh));
    }

    #[test]
    fn the_viewport_is_centered(res in resolution(), ww in 1u32..8000, wh in 1u32..8000) {
        let view = fitted(res, ww, wh);
        let right = ww - view.x() - view.width();
        let bottom = wh - view.y() - view.height();
        prop_assert!(view.x().abs_diff(right) <= 1);
        prop_assert!(view.y().abs_diff(bottom) <= 1);
    }

    #[test]
    fn a_window_at_least_as_big_gets_a_whole_scale(
        res in resolution(),
        extra_w in 0u32..4000,
        extra_h in 0u32..4000,
    ) {
        let view = fitted(res, res.width() + extra_w, res.height() + extra_h);
        let scale = view.width() / res.width();
        prop_assert!(scale >= 1);
        prop_assert_eq!(view.width(), res.width() * scale);
        prop_assert_eq!(view.height(), res.height() * scale);
    }
}

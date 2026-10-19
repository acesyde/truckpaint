//! The icon of the app. The picture is built into the program, so a download
//! shows it without any file next to it, and it is given to the window: the
//! taskbar or task switcher and the Dock show it.

use std::sync::Arc;

use egui::IconData;

/// The 512 px picture of `assets/icon/`: enough for the Dock at its largest and
/// for the sizes Windows and Linux ask for.
const PICTURE: &[u8] = include_bytes!("../../../assets/icon/truckpaint-512.png");

/// The icon of the window. `None`, with a warning in the log, when the built-in
/// picture cannot be read: the app then starts with the system's generic icon.
pub fn app_icon() -> Option<Arc<IconData>> {
    decode(PICTURE)
}

fn decode(png: &[u8]) -> Option<Arc<IconData>> {
    match eframe::icon_data::from_png_bytes(png) {
        Ok(icon) => Some(Arc::new(icon)),
        Err(err) => {
            tracing::warn!(%err, "the app icon cannot be read: starting without it");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 1024 px master, for the checks on the committed pictures.
    const MASTER: &[u8] = include_bytes!("../../../assets/icon/truckpaint-1024.png");

    fn alpha(rgba: &[u8], width: u32, x: u32, y: u32) -> u8 {
        rgba[((y * width + x) * 4 + 3) as usize]
    }

    #[test]
    fn the_built_in_picture_is_a_square_rgba_icon() {
        let icon = app_icon().expect("the built-in picture decodes");
        assert_eq!((icon.width, icon.height), (512, 512));
        assert_eq!(icon.rgba.len(), 512 * 512 * 4);
        let last = 511;
        for (x, y) in [(0, 0), (last, 0), (0, last), (last, last)] {
            assert_eq!(alpha(&icon.rgba, 512, x, y), 0, "corner {x},{y}");
        }
        assert!(
            alpha(&icon.rgba, 512, 256, 256) >= 250,
            "the middle is opaque"
        );
    }

    #[test]
    fn an_unreadable_picture_gives_no_icon() {
        assert!(decode(b"not a png").is_none());
        assert!(decode(&[]).is_none());
    }

    #[test]
    fn the_master_is_1024_with_transparent_corners_and_a_box_of_about_80_percent() {
        let master = image::load_from_memory(MASTER)
            .expect("the master is a PNG")
            .into_rgba8();
        let (width, height) = master.dimensions();
        assert_eq!((width, height), (1024, 1024));
        let alpha_at = |x: u32, y: u32| master.get_pixel(x, y)[3];
        for (x, y) in [(0, 0), (1023, 0), (0, 1023), (1023, 1023)] {
            assert_eq!(alpha_at(x, y), 0, "corner {x},{y}");
        }
        // The box of the opaque pixels (a fainter pixel is a soft shadow).
        let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
        for (x, y, pixel) in master.enumerate_pixels() {
            if pixel[3] >= 128 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x);
                bottom = bottom.max(y);
            }
        }
        let (box_width, box_height) = (right - left + 1, bottom - top + 1);
        let (low, high) = (1024 * 78 / 100, 1024 * 82 / 100);
        assert!(
            (low..=high).contains(&box_width) && (low..=high).contains(&box_height),
            "the box is {box_width} by {box_height} px"
        );
        let (center_x, center_y) = (left + right + 1, top + bottom + 1);
        assert!(
            center_x.abs_diff(1024) <= 4,
            "centered horizontally: {center_x}"
        );
        assert!(
            center_y.abs_diff(1024) <= 4,
            "centered vertically: {center_y}"
        );
    }
}

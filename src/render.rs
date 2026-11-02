//! Rasterizing icons: PNG export and previews drawn with half-block
//! characters in the terminal.
//!
//! Each terminal cell shows two vertical pixels: `▀` with the upper pixel as
//! foreground and the lower one as background. Single-color icons are drawn in
//! the terminal's own foreground color with no color codes, so they read on
//! light and dark themes alike.

use anyhow::{anyhow, Context, Result};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{ImageHrefResolver, Options, Tree};

/// Pixels with less coverage than this are left empty in previews.
const ALPHA_THRESHOLD: u8 = 110;

fn options() -> Options<'static> {
    Options {
        // Icon data is untrusted: never let an <image href> reach the disk or
        // decode embedded rasters.
        image_href_resolver: ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Options::default()
    }
}

/// Renders an SVG centered in a `size` x `size` transparent square.
pub fn rasterize(svg: &str, size: u32) -> Result<Pixmap> {
    let tree = Tree::from_str(svg, &options()).context("the SVG could not be parsed")?;
    let mut pixmap =
        Pixmap::new(size, size).ok_or_else(|| anyhow!("invalid raster size {size}"))?;
    let w = tree.size().width();
    let h = tree.size().height();
    let scale = size as f32 / w.max(h);
    let dx = (size as f32 - w * scale) / 2.0;
    let dy = (size as f32 - h * scale) / 2.0;
    resvg::render(
        &tree,
        Transform::from_row(scale, 0.0, 0.0, scale, dx, dy),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

pub fn to_png(svg: &str, size: u32) -> Result<Vec<u8>> {
    rasterize(svg, size)?
        .encode_png()
        .context("encoding the PNG failed")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    /// Shape only, in the terminal's foreground color. No escape codes.
    Mask,
    /// 24-bit colors from the icon.
    TrueColor,
}

/// Terminal lines for a rendered icon: `width` columns, `height / 2` rows.
pub fn terminal_lines(pixmap: &Pixmap, paint: Paint) -> Vec<String> {
    let w = pixmap.width();
    let h = pixmap.height();
    let px = |x: u32, y: u32| -> Option<(u8, u8, u8)> {
        if y >= h {
            return None;
        }
        let c = pixmap.pixel(x, y)?.demultiply();
        (c.alpha() >= ALPHA_THRESHOLD).then(|| (c.red(), c.green(), c.blue()))
    };
    let mut lines = Vec::new();
    for row in (0..h).step_by(2) {
        let mut line = String::new();
        for x in 0..w {
            let top = px(x, row);
            let bottom = px(x, row + 1);
            match paint {
                Paint::Mask => line.push(match (top, bottom) {
                    (Some(_), Some(_)) => '█',
                    (Some(_), None) => '▀',
                    (None, Some(_)) => '▄',
                    (None, None) => ' ',
                }),
                Paint::TrueColor => match (top, bottom) {
                    (Some((r, g, b)), Some((r2, g2, b2))) => line.push_str(&format!(
                        "\x1b[38;2;{r};{g};{b};48;2;{r2};{g2};{b2}m▀\x1b[0m"
                    )),
                    (Some((r, g, b)), None) => {
                        line.push_str(&format!("\x1b[38;2;{r};{g};{b}m▀\x1b[0m"))
                    }
                    (None, Some((r, g, b))) => {
                        line.push_str(&format!("\x1b[38;2;{r};{g};{b}m▄\x1b[0m"))
                    }
                    (None, None) => line.push(' '),
                },
            }
        }
        lines.push(line);
    }
    lines
}

/// Lays out previews side by side, each with a number label underneath, as
/// many per row as fit in `width` columns.
pub fn grid(cells: &[Vec<String>], cell_width: usize, width: usize) -> String {
    let gap = 2;
    let per_row = (width.max(cell_width) + gap) / (cell_width + gap);
    let mut out = String::new();
    for (row_index, row) in cells.chunks(per_row.max(1)).enumerate() {
        let height = row.iter().map(Vec::len).max().unwrap_or(0);
        for y in 0..height {
            let mut line = String::new();
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    line.push_str(&" ".repeat(gap));
                }
                match cell.get(y) {
                    Some(s) => line.push_str(s),
                    None => line.push_str(&" ".repeat(cell_width)),
                }
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
        let mut labels = String::new();
        for i in 0..row.len() {
            if i > 0 {
                labels.push_str(&" ".repeat(gap));
            }
            let n = row_index * per_row + i + 1;
            labels.push_str(&format!("{:^cell_width$}", format!("[{n}]")));
        }
        out.push_str(labels.trim_end());
        out.push_str("\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8" width="8" height="8"><rect x="0" y="0" width="4" height="8" fill="currentColor"/></svg>"#;
    const RED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2 2"><rect width="2" height="1" fill="#ff0000"/></svg>"##;

    #[test]
    fn mask_draws_the_shape() {
        let pix = rasterize(SQUARE, 8).unwrap();
        let lines = terminal_lines(&pix, Paint::Mask);
        assert_eq!(lines.len(), 4);
        for line in &lines {
            assert_eq!(line, "████    ");
        }
    }

    #[test]
    fn truecolor_uses_icon_colors() {
        let pix = rasterize(RED, 2).unwrap();
        let lines = terminal_lines(&pix, Paint::TrueColor);
        assert_eq!(
            lines,
            vec!["\x1b[38;2;255;0;0m▀\x1b[0m\x1b[38;2;255;0;0m▀\x1b[0m"]
        );
    }

    #[test]
    fn wide_icons_are_centered() {
        let wide = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 4"><rect width="8" height="4"/></svg>"#;
        let pix = rasterize(wide, 8).unwrap();
        let lines = terminal_lines(&pix, Paint::Mask);
        assert_eq!(lines, vec!["        ", "████████", "████████", "        "]);
    }

    #[test]
    fn png_has_signature() {
        let png = to_png(SQUARE, 32).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn external_images_are_not_loaded() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 4 4"><image href="/etc/hosts" width="4" height="4"/></svg>"#;
        let pix = rasterize(svg, 4).unwrap();
        assert!(pix.pixels().iter().all(|p| p.alpha() == 0));
    }

    #[test]
    fn grid_wraps_and_labels() {
        let cell = vec!["ab".to_string()];
        let out = grid(&[cell.clone(), cell.clone(), cell], 2, 7);
        assert_eq!(out, "ab  ab\n[1]  [2]\n\nab\n[3]\n\n");
    }

    #[test]
    fn invalid_svg_is_an_error() {
        assert!(rasterize("<svg", 8).is_err());
    }
}

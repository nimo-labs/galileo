#[cfg(not(target_arch = "wasm32"))]
use std::ops::Deref;
use std::sync::Arc;

use galileo_types::cartesian::{Point3, Vector2};
use galileo_types::geometry::Geom;
use galileo_types::MultiPoint;
use image::EncodableLayout;

use crate::decoded_image::DecodedImage;
use crate::error::GalileoError;
use crate::layer::feature_layer::symbol::Symbol;
use crate::render::point_paint::{MarkerStyle, PointPaint};
use crate::render::render_bundle::RenderBundle;
use crate::render::text::TextStyle;
use crate::Color;

/// Renders a point as a circle of fixes size.
#[derive(Debug, Copy, Clone)]
pub struct CirclePointSymbol {
    /// Color of the circle.
    pub color: Color,
    /// Diameter of the circle.
    pub size: f64,
}

impl CirclePointSymbol {
    /// Create a new instance.
    pub fn new(color: Color, size: f64) -> Self {
        Self { color, size }
    }
}

impl<F> Symbol<F> for CirclePointSymbol {
    fn render(
        &self,
        _feature: &F,
        geometry: &Geom<Point3>,
        min_resolution: f64,
        bundle: &mut RenderBundle,
    ) {
        let paint = PointPaint::circle(self.color, self.size as f32);
        match geometry {
            Geom::Point(point) => {
                bundle.add_point(point, &paint, min_resolution);
            }
            Geom::MultiPoint(points) => {
                points.iter_points().for_each(|p| {
                    bundle.add_point(&p, &paint, min_resolution);
                });
            }
            _ => {}
        }
    }
}

/// Symbol that renders a point with an image. The image size is fixed on the screen and does not depend on map
/// resolution.
pub struct ImagePointSymbol {
    image: Arc<DecodedImage>,
    offset: Vector2<f32>,
    scale: f32,
}

impl ImagePointSymbol {
    /// Loads the image from the file system path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_path(path: &str, offset: Vector2<f32>, scale: f32) -> Result<Self, GalileoError> {
        use galileo_types::cartesian::Size;

        let image = image::io::Reader::open(path)?
            .decode()
            .map_err(|_| GalileoError::ImageDecode)?;

        Ok(Self {
            image: Arc::new(DecodedImage::from_raw(
                Vec::from(image.to_rgba8().deref()),
                Size::new(image.width(), image.height()),
            )?),
            offset,
            scale,
        })
    }

    /// Decodes the image from the raw bytes.
    pub fn from_bytes(data: &[u8], offset: Vector2<f32>, scale: f32) -> Result<Self, GalileoError> {
        use galileo_types::cartesian::Size;

        let image = image::load_from_memory(data)
            .map_err(|_| GalileoError::ImageDecode)?
            .to_rgba8();

        Ok(Self {
            image: Arc::new(DecodedImage::from_raw(
                Vec::from(image.as_bytes()),
                Size::new(image.width(), image.height()),
            )?),
            offset,
            scale,
        })
    }
}

impl<F> Symbol<F> for ImagePointSymbol {
    fn render(
        &self,
        _feature: &F,
        geometry: &Geom<Point3>,
        _min_resolution: f64,
        bundle: &mut RenderBundle,
    ) {
        let add_marker = |point: &Point3, bundle: &mut RenderBundle| {
            bundle.add_marker(
                point,
                &MarkerStyle::Image {
                    image: self.image.clone(),
                    anchor: self.offset,
                    size: Some((self.image.size().cast::<f32>() * self.scale).cast()),
                },
            );
        };

        match geometry {
            Geom::Point(point) => add_marker(point, bundle),
            Geom::MultiPoint(points) => points.iter_points().for_each(|point| {
                add_marker(&point, bundle);
            }),
            _ => {}
        }
    }
}

/// Symbol that renders text with a black rectangular background and white text.
/// The background size is calculated automatically to fit the text.
#[derive(Debug, Clone)]
pub struct TextMarkerSymbol {
    /// Text style for font, size, and alignment.
    pub text_style: TextStyle,
    /// Padding around the text in pixels.
    pub padding: f32,
}

/// Trait for features that can provide text for TextMarkerSymbol
pub trait TextProvider {
    /// Returns the text to display in the marker
    fn get_text(&self) -> &str;
}

impl TextMarkerSymbol {
    /// Create a new text marker with default styling.
    pub fn new() -> Self {
        Self {
            text_style: TextStyle {
                font_family: vec!["DejaVu Sans".to_string()],
                font_size: 12.0,
                font_color: Color::WHITE,
                horizontal_alignment: crate::render::text::HorizontalAlignment::Center,
                vertical_alignment: crate::render::text::VerticalAlignment::Middle,
                weight: crate::render::text::FontWeight::NORMAL,
                style: crate::render::text::FontStyle::Normal,
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
            },
            padding: 4.0,
        }
    }

    /// Create a new text marker with custom text style.
    pub fn with_style(text_style: TextStyle) -> Self {
        Self {
            text_style,
            padding: 4.0,
        }
    }

    /// Set the padding around the text.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl<F: TextProvider> Symbol<F> for TextMarkerSymbol {
    fn render(
        &self,
        feature: &F,
        geometry: &Geom<Point3>,
        min_resolution: f64,
        bundle: &mut RenderBundle,
    ) {
        let text = feature.get_text();
        let render_text_marker = |point: &Point3, bundle: &mut RenderBundle| {
            // First render a black rectangle as background
            // We'll estimate the text size based on font size and character count
            let estimated_char_width = self.text_style.font_size * 0.6;
            let text_width = estimated_char_width * text.len() as f32;
            let text_height = self.text_style.font_size;
            
            let bg_width = text_width + (self.padding * 2.0);
            let bg_height = text_height + (self.padding * 2.0);
            
            // Render black rectangle background
            let bg_paint = PointPaint::square(Color::BLACK, bg_width.max(bg_height));
            bundle.add_point(point, &bg_paint, min_resolution);
            
            // Render white text on top
            let text_paint = PointPaint::label(text, &self.text_style);
            bundle.add_point(point, &text_paint, min_resolution);
        };

        match geometry {
            Geom::Point(point) => render_text_marker(point, bundle),
            Geom::MultiPoint(points) => points.iter_points().for_each(|point| {
                render_text_marker(&point, bundle);
            }),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_symbol_from_file() {
        let symbol = ImagePointSymbol::from_path(
            "examples/data/pin-yellow.png",
            Vector2::new(0.5, 1.0),
            1.0,
        )
        .unwrap();
        assert_eq!(symbol.image.width(), 62);
        assert_eq!(symbol.image.height(), 99);
        assert_eq!(symbol.image.byte_size(), 62 * 99 * 4);
    }
}

//! [`RenderBundle`] is used to store primitives and prepare them for rendering with the rendering backend.

use std::sync::Arc;

use galileo_types::cartesian::{CartesianPoint3d, Point2, Vector2};
use galileo_types::contour::Contour;
use galileo_types::Polygon;
use num_traits::AsPrimitive;
use screen_set::ScreenRenderSet;
use serde::{Deserialize, Serialize};

use super::point_paint::MarkerStyle;
use super::text::TextStyle;
use crate::decoded_image::DecodedImage;
use crate::render::point_paint::PointPaint;
use crate::render::render_bundle::world_set::WorldRenderSet;
use crate::render::{ImagePaint, LinePaint, PolygonPaint};

pub(crate) mod screen_set;
pub(crate) mod world_set;

/// Render bundle is used to store render primitives and prepare them to be rendered with the rendering backend.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderBundle {
    pub(crate) world_set: WorldRenderSet,
    pub(crate) screen_sets: Vec<ScreenRenderSet>,
}

impl RenderBundle {
    /// Adds an image to the bundle.
    pub fn add_image(
        &mut self,
        image: Arc<DecodedImage>,
        vertices: [Point2; 4],
        paint: ImagePaint,
    ) {
        self.world_set.add_image(image, vertices, paint);
    }
    ///
    /// Adds an image to the bundle.
    pub fn add_image_owned(
        &mut self,
        image: DecodedImage,
        vertices: [Point2; 4],
        paint: ImagePaint,
    ) {
        self.world_set.add_image_owned(image, vertices, paint);
    }

    /// Adds a point to the bundle.
    pub fn add_point<N, P>(&mut self, point: &P, paint: &PointPaint, _min_resolution: f64)
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N>,
    {
        self.world_set.add_point(point, paint);
    }

    /// Adds a line to the bundle.
    pub fn add_line<N, P, C>(&mut self, line: &C, paint: &LinePaint, min_resolution: f64)
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N>,
        C: Contour<Point = P>,
    {
        self.world_set.add_line(line, paint, min_resolution);
    }

    /// Adds a polygon to the bundle.
    pub fn add_polygon<N, P, Poly>(
        &mut self,
        polygon: &Poly,
        paint: &PolygonPaint,
        min_resolution: f64,
    ) where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N>,
        Poly: Polygon,
        Poly::Contour: Contour<Point = P>,
    {
        self.world_set.add_polygon(polygon, paint, min_resolution);
    }

    /// Adds a label to the bundle.
    pub fn add_label<N, P>(
        &mut self,
        position: &P,
        text: &str,
        style: &TextStyle,
        offset: Vector2<f32>,
        attach_to_map: bool,
    ) where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N>,
    {
        if attach_to_map {
            self.world_set.add_label(position, text, style, offset);
        } else if let Some(set) = ScreenRenderSet::new_from_label(position, text, style, offset) {
            self.screen_sets.push(set);
        }
    }

    /// Adds a marker to the bundle.
    pub fn add_marker<N, P>(&mut self, position: &P, style: &MarkerStyle)
    where
        N: AsPrimitive<f32>,
        P: CartesianPoint3d<Num = N>,
    {
        if let Some(set) = ScreenRenderSet::new_from_marker(position, style) {
            self.screen_sets.push(set);
        }
    }
}

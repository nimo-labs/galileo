//! Text markers with black background

use galileo::layer::feature_layer::{Feature, TextMarkerSymbol, TextProvider};
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use galileo::layer::FeatureLayer;
use galileo::{Map, MapBuilder};
use galileo_egui::{EguiMap, EguiMapState};
use galileo_types::geo::impls::GeoPoint2d;
use galileo_types::geo::Crs;
use galileo_types::geometry_type::GeoSpace2d;
use galileo_types::latlon;
use parking_lot::RwLock;
use std::sync::Arc;

struct EguiMapApp {
    map: EguiMapState,
    feature_layer: Arc<RwLock<FeatureLayer<GeoPoint2d, TextMarker, TextMarkerSymbol, GeoSpace2d>>>,
}

impl EguiMapApp {
    fn new(mut egui_map_state: EguiMapState) -> Self {
        let markers = vec![
            TextMarker {
                position: latlon!(37.566, 126.9784),
                text: "Seoul".to_string(),
            },
            TextMarker {
                position: latlon!(40.7128, -74.0060),
                text: "New York".to_string(),
            },
            TextMarker {
                position: latlon!(51.5074, -0.1278),
                text: "London".to_string(),
            },
            TextMarker {
                position: latlon!(35.6762, 139.6503),
                text: "Tokyo".to_string(),
            },
            TextMarker {
                position: latlon!(48.8566, 2.3522),
                text: "Paris".to_string(),
            },
        ];

        let layer = FeatureLayer::new(markers, TextMarkerSymbol::new(), Crs::WGS84);
        let layer = Arc::new(RwLock::new(layer));

        egui_map_state.map_mut().layers_mut().push(layer.clone());

        Self {
            map: egui_map_state,
            feature_layer: layer,
        }
    }
}

impl eframe::App for EguiMapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            EguiMap::new(&mut self.map).show_ui(ui);

            egui::Window::new("Text Markers Demo").show(ctx, |ui| {
                ui.label("Text markers with black background");
                ui.label("White text on black rectangular background");
                ui.label("Click and drag to pan the map");
                ui.label("Scroll to zoom");
            });
        });
    }
}

fn main() {
    run()
}

pub(crate) fn run() {
    let map = create_map();
    galileo_egui::InitBuilder::new(map)
        .with_app_builder(|egui_map_state| Box::new(EguiMapApp::new(egui_map_state)))
        .init()
        .expect("failed to initialize");
}

fn create_map() -> Map {
    let layer = RasterTileLayerBuilder::new_osm()
        .build()
        .expect("failed to create layer");

    MapBuilder::default()
        .with_layer(layer)
        .with_latlon(37.566, 126.9784)
        .with_z_level(3)
        .build()
}

struct TextMarker {
    position: GeoPoint2d,
    text: String,
}

impl Feature for TextMarker {
    type Geom = GeoPoint2d;

    fn geometry(&self) -> &Self::Geom {
        &self.position
    }
}

impl TextProvider for TextMarker {
    fn get_text(&self) -> &str {
        &self.text
    }
}

// TextMarkerSymbol uses the generic implementation from the library
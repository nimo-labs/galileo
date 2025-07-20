//! Example demonstrating dynamic tile provider functionality.
//!
//! This example shows how to use the dynamic URL tile loaders that allow the host application
//! to provide URLs and parameters to force Galileo to use new map tiles.

use galileo::layer::raster_tile_layer::{DynamicUrlTileLoader, RasterTileLayerBuilder};
use galileo::layer::vector_tile_layer::{style::VectorTileStyle, VectorTileLayerBuilder};

/// Example demonstrating dynamic raster tile provider
fn raster_tile_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a dynamic URL tile loader
    let loader = DynamicUrlTileLoader::new(
        "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
        None,
        false,
    );

    // Create a layer with the dynamic loader
    let _layer = RasterTileLayerBuilder::new_with_loader(loader)
        .with_attribution(
            "© OpenStreetMap contributors".to_string(),
            "https://www.openstreetmap.org/copyright".to_string(),
        )
        .build()?;

    println!("Raster tile layer created with dynamic URL loader");

    Ok(())
}

/// Example demonstrating dynamic vector tile provider
fn vector_tile_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a layer with dynamic URL vector tile loader
    let _layer = VectorTileLayerBuilder::new_dynamic_url(
        "https://vector.tiles.mapbox.com/v4/mapbox.mapbox-streets-v8/{z}/{x}/{y}.vector.pbf",
    )
    .with_style(VectorTileStyle::default())
    .with_attribution(
        "© Mapbox".to_string(),
        "https://www.mapbox.com/about/maps/".to_string(),
    )
    .build()?;

    println!("Vector tile layer created with dynamic URL loader");

    Ok(())
}

/// Example showing how to update tile URLs and parameters at runtime
fn runtime_update_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a dynamic loader
    let loader = DynamicUrlTileLoader::new(
        "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
        None,
        false,
    );

    // Add some parameters
    loader.add_parameter("format", "png");
    loader.add_parameter("quality", "high");

    // Update the URL template
    loader.update_url_template("https://custom.tiles.com/{z}/{x}/{y}.png");

    // Update parameters
    loader.update_parameters(vec![
        ("api_key".to_string(), "your_api_key".to_string()),
        ("style".to_string(), "dark".to_string()),
        ("format".to_string(), "png".to_string()),
    ]);

    // Remove a parameter
    loader.remove_parameter("quality");

    // Clear all parameters
    loader.clear_parameters();

    println!("Dynamic loader configured and updated");

    Ok(())
}

/// Example showing how to use different tile sources
fn multiple_sources_example() -> Result<(), Box<dyn std::error::Error>> {
    // OpenStreetMap tiles
    let osm_loader = DynamicUrlTileLoader::new(
        "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
        None,
        false,
    );

    // CartoDB tiles
    let cartodb_loader = DynamicUrlTileLoader::new(
        "https://cartodb-basemaps-a.global.ssl.fastly.net/light_all/{z}/{x}/{y}.png",
        None,
        false,
    );

    // Stamen tiles
    let stamen_loader = DynamicUrlTileLoader::new(
        "https://stamen-tiles.a.ssl.fastly.net/terrain/{z}/{x}/{y}.png",
        None,
        false,
    );

    // Create layers with different sources
    let _osm_layer = RasterTileLayerBuilder::new_with_loader(osm_loader)
        .with_attribution(
            "© OpenStreetMap contributors".to_string(),
            "https://www.openstreetmap.org/copyright".to_string(),
        )
        .build()?;

    let _cartodb_layer = RasterTileLayerBuilder::new_with_loader(cartodb_loader)
        .with_attribution(
            "© CartoDB".to_string(),
            "https://cartodb.com/attributions".to_string(),
        )
        .build()?;

    let _stamen_layer = RasterTileLayerBuilder::new_with_loader(stamen_loader)
        .with_attribution(
            "© Stamen Design".to_string(),
            "https://stamen.com/".to_string(),
        )
        .build()?;

    println!("Created layers with different tile sources");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Dynamic Tile Provider Examples");
    println!("==============================");

    println!("\n1. Raster Tile Example:");
    raster_tile_example()?;

    println!("\n2. Vector Tile Example:");
    vector_tile_example()?;

    println!("\n3. Runtime Update Example:");
    runtime_update_example()?;

    println!("\n4. Multiple Sources Example:");
    multiple_sources_example()?;

    println!("\nAll examples completed successfully!");

    Ok(())
}

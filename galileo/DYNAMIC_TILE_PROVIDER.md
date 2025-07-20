# Dynamic Tile Provider

This document describes the dynamic tile provider functionality in Galileo, which allows host applications to provide URLs and parameters to force Galileo to use new map tiles.

## Overview

The dynamic tile provider functionality consists of two main components:

1. **DynamicUrlTileLoader** - For raster tiles
2. **DynamicUrlVtLoader** - For vector tiles

These loaders allow you to:
- Update tile URLs at runtime
- Add, remove, and modify URL parameters
- Switch between different tile sources dynamically
- Maintain caching functionality

## Raster Tile Loader

### Basic Usage

```rust
use galileo::layer::raster_tile_layer::{DynamicUrlTileLoader, RasterTileLayerBuilder};

// Create a dynamic loader
let loader = DynamicUrlTileLoader::new(
    "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    None,  // cache
    false  // offline_mode
);

// Create a layer with the loader
let layer = RasterTileLayerBuilder::new_with_loader(loader)
    .with_file_cache_checked("./tile_cache")
    .with_attribution(
        "© OpenStreetMap contributors".to_string(),
        "https://www.openstreetmap.org/copyright".to_string(),
    )
    .build()?;
```

### Convenience Builder Method

```rust
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;

// Create a layer with dynamic URL loader using the convenience method
let layer = RasterTileLayerBuilder::new_dynamic_url(
    "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
)
.with_file_cache_checked("./tile_cache")
.with_attribution(
    "© OpenStreetMap contributors".to_string(),
    "https://www.openstreetmap.org/copyright".to_string(),
)
.build()?;
```

### Runtime URL Updates

```rust
// Update the URL template
loader.update_url_template("https://custom.tiles.com/{z}/{x}/{y}.png");

// Add parameters
loader.add_parameter("api_key", "your_api_key");
loader.add_parameter("style", "dark");

// Update all parameters at once
loader.update_parameters(vec![
    ("api_key".to_string(), "your_api_key".to_string()),
    ("style".to_string(), "dark".to_string()),
    ("format".to_string(), "png".to_string())
]);

// Remove a parameter
loader.remove_parameter("style");

// Clear all parameters
loader.clear_parameters();
```

## Vector Tile Loader

### Basic Usage

```rust
use galileo::layer::vector_tile_layer::tile_provider::loader::DynamicUrlVtLoader;
use galileo::layer::vector_tile_layer::{VectorTileLayerBuilder, style::VectorTileStyle};

// Create a dynamic vector tile loader
let loader = DynamicUrlVtLoader::new(
    "https://vector.tiles.mapbox.com/v4/mapbox.mapbox-streets-v8/{z}/{x}/{y}.vector.pbf",
    None,  // cache
    false  // offline_mode
);

// Create a layer with the loader
let layer = VectorTileLayerBuilder::new_with_provider(provider)
    .with_file_cache_checked("./vector_tile_cache")
    .with_style(VectorTileStyle::default())
    .with_attribution(
        "© Mapbox".to_string(),
        "https://www.mapbox.com/about/maps/".to_string(),
    )
    .build()?;
```

### Convenience Builder Method

```rust
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;

// Create a layer with dynamic URL vector tile loader
let layer = VectorTileLayerBuilder::new_dynamic_url(
    "https://vector.tiles.mapbox.com/v4/mapbox.mapbox-streets-v8/{z}/{x}/{y}.vector.pbf"
)
.with_file_cache_checked("./vector_tile_cache")
.with_style(VectorTileStyle::default())
.with_attribution(
    "© Mapbox".to_string(),
    "https://www.mapbox.com/about/maps/".to_string(),
)
.build()?;
```

### Runtime URL Updates

The vector tile loader supports the same runtime update methods as the raster tile loader:

```rust
// Update the URL template
loader.update_url_template("https://custom.vector.tiles.com/{z}/{x}/{y}.pbf");

// Add parameters
loader.add_parameter("access_token", "your_access_token");
loader.add_parameter("style", "streets-v11");

// Update all parameters at once
loader.update_parameters(vec![
    ("access_token".to_string(), "your_access_token".to_string()),
    ("style".to_string(), "streets-v11".to_string())
]);

// Remove a parameter
loader.remove_parameter("style");

// Clear all parameters
loader.clear_parameters();
```

## URL Template Format

Both loaders use a URL template format with placeholders for tile coordinates:

- `{z}` - Zoom level
- `{x}` - Tile X coordinate
- `{y}` - Tile Y coordinate

### Examples

```
https://tile.openstreetmap.org/{z}/{x}/{y}.png
https://vector.tiles.mapbox.com/v4/mapbox.mapbox-streets-v8/{z}/{x}/{y}.vector.pbf
https://api.maptiler.com/tiles/v3-openmaptiles/{z}/{x}/{y}.pbf?key=YOUR_API_KEY
```

## Parameters

Parameters are added as query string parameters to the URL. The loaders automatically handle URL encoding of parameter names and values.

### Parameter Examples

```rust
// Single parameters
loader.add_parameter("api_key", "your_api_key");
loader.add_parameter("style", "dark");

// Multiple parameters
loader.update_parameters(vec![
    ("api_key".to_string(), "your_api_key".to_string()),
    ("style".to_string(), "dark".to_string()),
    ("format".to_string(), "png".to_string())
]);
```

This will generate URLs like:
```
https://tiles.example.com/10/512/256.png?api_key=your_api_key&style=dark&format=png
```

## Caching

Both loaders support caching through the `PersistentCacheController` trait:

```rust
use galileo::layer::data_provider::FileCacheController;

// Create a file cache
let cache = FileCacheController::new("./tile_cache")?;

// Create loader with cache
let loader = DynamicUrlTileLoader::new(
    "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    Some(Box::new(cache)),
    false
);
```

## Offline Mode

Both loaders support offline mode, which prevents network requests and only uses cached tiles:

```rust
let loader = DynamicUrlTileLoader::new(
    "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    Some(Box::new(cache)),
    true  // offline_mode = true
);
```

## Error Handling

The loaders handle various error conditions:

- **Network errors** - When tiles cannot be downloaded
- **Missing tiles** - When a tile doesn't exist on the server
- **Decoding errors** - When tile data cannot be decoded
- **Cache errors** - When cache operations fail

## Best Practices

1. **Use caching** - Always use file caching for better performance
2. **Handle errors gracefully** - Implement proper error handling for tile loading failures
3. **Update URLs carefully** - Ensure new URLs follow the same tile schema
4. **Monitor usage** - Be aware of rate limits and usage quotas for tile services
5. **Use HTTPS** - Always use HTTPS URLs for security

## Examples

See the `examples/dynamic_tile_provider.rs` file for complete working examples of how to use the dynamic tile provider functionality.

## API Reference

### DynamicUrlTileLoader

- `new(url_template, cache, offline_mode)` - Create a new loader
- `update_url_template(template)` - Update the URL template
- `add_parameter(key, value)` - Add a single parameter
- `update_parameters(parameters)` - Update all parameters
- `remove_parameter(key)` - Remove a parameter by key
- `clear_parameters()` - Clear all parameters

### DynamicUrlVtLoader

- `new(url_template, cache, offline_mode)` - Create a new loader
- `update_url_template(template)` - Update the URL template
- `add_parameter(key, value)` - Add a single parameter
- `update_parameters(parameters)` - Update all parameters
- `remove_parameter(key)` - Remove a parameter by key
- `clear_parameters()` - Clear all parameters 
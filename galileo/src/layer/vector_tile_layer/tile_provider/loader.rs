//! Vector tile loader stuff.

use bytes::Bytes;
use galileo_mvt::MvtTile;
use maybe_sync::{MaybeSend, MaybeSync};
use std::sync::Arc;

use crate::error::GalileoError;
use crate::layer::data_provider::{PersistentCacheController, UrlSource};
use crate::platform::PlatformService;
use crate::tile_schema::TileIndex;

/// Error that can occur when trying to load a vector tile.
pub enum TileLoadError {
    /// Could not connect to the remote server.
    Network,
    /// Tile with the given index does not exist.
    DoesNotExist,
    /// Failed to decode vector tile from the binary data.
    Decoding,
}

/// Loader for vector tiles.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait VectorTileLoader: MaybeSend + MaybeSync {
    /// Load tile with the given index.
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError>;
}

/// Load the tile from the Web.
pub struct WebVtLoader {
    cache: Option<Box<dyn PersistentCacheController<str, Bytes>>>,
    url_source: Box<dyn UrlSource<TileIndex>>,
    offline_mode: bool,
}

impl WebVtLoader {
    /// Create a new instance.
    pub fn new(
        cache: Option<Box<dyn PersistentCacheController<str, Bytes>>>,
        url_source: impl UrlSource<TileIndex> + 'static,
        offline_mode: bool,
    ) -> Self {
        Self {
            cache,
            url_source: Box::new(url_source),
            offline_mode,
        }
    }

    async fn load_raw(&self, url: &str) -> Result<Bytes, TileLoadError> {
        if let Some(data) = self.cache.as_ref().and_then(|cache| cache.get(url)) {
            log::trace!("Cache hit for url {url}");
            return Ok(data);
        }

        if self.offline_mode {
            return Err(TileLoadError::DoesNotExist);
        }

        let bytes = crate::platform::instance()
            .load_bytes_from_url(url)
            .await
            .map_err(|err| match err {
                GalileoError::NotFound => TileLoadError::DoesNotExist,
                _ => TileLoadError::Network,
            })?;

        log::info!("Loaded tile from url: {url}");

        if let Some(cache) = &self.cache {
            if let Err(error) = cache.insert(url, &bytes) {
                log::warn!("Failed to write persistent cache entry: {error:?}");
            }
        }

        Ok(bytes)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl VectorTileLoader for WebVtLoader {
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError> {
        let url = (self.url_source)(&index);

        log::trace!("Loading tile {index:?} from url {url}");
        let bytes = self.load_raw(&url).await?;

        log::trace!("Tile {index:?} loaded. Byte size: {}", bytes.len());

        let mvt = MvtTile::decode(bytes, false).map_err(|_| TileLoadError::Decoding)?;

        log::trace!("Tile {index:?} successfully decoded");

        Ok(mvt)
    }
}

/// Dynamic URL vector tile loader that allows the host application to provide URLs and parameters
/// to force Galileo to use new vector map tiles.
///
/// This loader supports dynamic URL configuration and can be updated with new URLs and parameters
/// at runtime. It maintains a cache of loaded tiles and can be configured with various parameters.
///
/// # Example
///
/// ```no_run
/// use galileo::layer::vector_tile_layer::tile_provider::loader::{VectorTileLoader, DynamicUrlVtLoader};
/// use galileo::tile_schema::TileIndex;
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// let loader = DynamicUrlVtLoader::new(
///     "https://vector.tiles.com/{z}/{x}/{y}.pbf",
///     None,
///     false
/// );
///
/// // Update the URL template and parameters
/// loader.update_url_template("https://custom.vector.tiles.com/{z}/{x}/{y}.pbf");
/// loader.update_parameters(vec![("api_key", "your_api_key"), ("style", "dark")]);
///
/// # tokio_test::block_on(async {
/// let tile = loader.load(TileIndex::new(3, 5, 3)).await.expect("failed to load tile");
/// # });
/// ```
pub struct DynamicUrlVtLoader {
    url_template: Arc<parking_lot::RwLock<String>>,
    parameters: Arc<parking_lot::RwLock<Vec<(String, String)>>>,
    cache: Option<Box<dyn PersistentCacheController<str, Bytes>>>,
    offline_mode: bool,
}

impl DynamicUrlVtLoader {
    /// Creates a new instance of the dynamic URL vector tile loader.
    pub fn new(
        url_template: impl Into<String>,
        cache: Option<Box<dyn PersistentCacheController<str, Bytes>>>,
        offline_mode: bool,
    ) -> Self {
        Self {
            url_template: Arc::new(parking_lot::RwLock::new(url_template.into())),
            parameters: Arc::new(parking_lot::RwLock::new(Vec::new())),
            cache,
            offline_mode,
        }
    }

    /// Updates the URL template used for generating tile URLs.
    ///
    /// The template should contain placeholders for {z}, {x}, and {y} coordinates.
    /// Example: "https://vector.tiles.com/{z}/{x}/{y}.pbf"
    pub fn update_url_template(&self, new_template: impl Into<String>) {
        *self.url_template.write() = new_template.into();
    }

    /// Updates the parameters that will be appended to the URL as query parameters.
    ///
    /// Parameters are added as key=value pairs in the URL query string.
    pub fn update_parameters(&self, new_parameters: Vec<(String, String)>) {
        *self.parameters.write() = new_parameters;
    }

    /// Adds a single parameter to the existing parameters.
    pub fn add_parameter(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut params = self.parameters.write();
        params.push((key.into(), value.into()));
    }

    /// Removes a parameter by key.
    pub fn remove_parameter(&self, key: &str) {
        let mut params = self.parameters.write();
        params.retain(|(k, _)| k != key);
    }

    /// Clears all parameters.
    pub fn clear_parameters(&self) {
        self.parameters.write().clear();
    }

    /// Generates the URL for a given tile index using the current template and parameters.
    fn generate_url(&self, index: &TileIndex) -> String {
        let template = self.url_template.read();
        let params = self.parameters.read();

        let mut url = template
            .replace("{z}", &index.z.to_string())
            .replace("{x}", &index.x.to_string())
            .replace("{y}", &index.y.to_string());

        if !params.is_empty() {
            let query_string: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            url.push_str("?");
            url.push_str(&query_string);
        }

        url
    }

    async fn load_raw(&self, url: &str) -> Result<Bytes, TileLoadError> {
        if let Some(data) = self.cache.as_ref().and_then(|cache| cache.get(url)) {
            log::trace!("Cache hit for url {url}");
            return Ok(data);
        }

        if self.offline_mode {
            return Err(TileLoadError::DoesNotExist);
        }

        let bytes = crate::platform::instance()
            .load_bytes_from_url(url)
            .await
            .map_err(|err| match err {
                crate::error::GalileoError::NotFound => TileLoadError::DoesNotExist,
                _ => TileLoadError::Network,
            })?;

        log::info!("Loaded tile from url: {url}");

        if let Some(cache) = &self.cache {
            if let Err(error) = cache.insert(url, &bytes) {
                log::warn!("Failed to write persistent cache entry: {error:?}");
            }
        }

        Ok(bytes)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl VectorTileLoader for DynamicUrlVtLoader {
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError> {
        let url = self.generate_url(&index);

        log::trace!("Loading tile {index:?} from url {url}");
        let bytes = self.load_raw(&url).await?;

        log::trace!("Tile {index:?} loaded. Byte size: {}", bytes.len());

        let mvt = MvtTile::decode(bytes, false).map_err(|_| TileLoadError::Decoding)?;

        log::trace!("Tile {index:?} successfully decoded");

        Ok(mvt)
    }
}

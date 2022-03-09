use geos::{CoordSeq, GResult, Geom, GeometryTypes};
use std::collections::HashMap;

// FeatureRenderer.java:
//
// private void renderLineOrPolygon(FeatureCollector.Feature feature, Geometry input) {
//   long id = idGenerator.incrementAndGet();
//   boolean area = input instanceof Polygonal;
//   double worldLength = (area || input.getNumGeometries() > 1) ? 0 : input.getLength();
//   String numPointsAttr = feature.getNumPointsAttr();
//   for (int z = feature.getMaxZoom(); z >= feature.getMinZoom(); z--) {
//     double scale = 1 << z;
//     double tolerance = feature.getPixelToleranceAtZoom(z) / 256d;
//     double minSize = feature.getMinPixelSizeAtZoom(z) / 256d;
//     if (area) {
//       // treat minPixelSize as the edge of a square that defines minimum area for features
//       minSize *= minSize;
//     } else if (worldLength > 0 && worldLength * scale < minSize) {
//       // skip linestring, too short
//       continue;
//     }

//     // TODO potential optimization: iteratively simplify z+1 to get z instead of starting with original geom each time
//     // simplify only takes 4-5 minutes of wall time when generating the planet though, so not a big deal
//     Geometry geom = AffineTransformation.scaleInstance(scale, scale).transform(input);
//     geom = DouglasPeuckerSimplifier.simplify(geom, tolerance);

//     List<List<CoordinateSequence>> groups = GeometryCoordinateSequences.extractGroups(geom, minSize);
//     double buffer = feature.getBufferPixelsAtZoom(z) / 256;
//     TileExtents.ForZoom extents = config.bounds().tileExtents().getForZoom(z);
//     TiledGeometry sliced = TiledGeometry.sliceIntoTiles(groups, buffer, area, z, extents, feature.getSourceId());
//     Map<String, Object> attrs = feature.getAttrsAtZoom(sliced.zoomLevel());
//     if (numPointsAttr != null) {
//       // if profile wants the original number of points that the simplified but untiled geometry started with
//       attrs = new HashMap<>(attrs);
//       attrs.put(numPointsAttr, geom.getNumPoints());
//     }
//     writeTileFeatures(z, id, feature, sliced, attrs);
//   }

//   stats.processedElement(area ? "polygon" : "line", feature.getLayer());
// }

/// A builder for an output map feature that contains all the information that will be needed to render vector tile
/// features from the input element.
///
/// Some feature attributes are set globally (like sort key), and some allow the value to change by zoom-level (like
/// tags).
pub struct Feature {
    /// Original ID of the source feature that this feature came from (i.e. OSM node/way ID).
    pub source_id: u64,
    pub min_zoom: u16,
    pub max_zoom: u16,
    // private static final double DEFAULT_LABEL_GRID_SIZE = 0;
    // private static final int DEFAULT_LABEL_GRID_LIMIT = 0;

    // private final String layer;
    // private final Geometry geom;
    // private final Map<String, Object> attrs = new TreeMap<>();
    // private final GeometryType geometryType;
    // private final long sourceId;

    // private int sortKey = 0;

    // private int minzoom = config.minzoom();
    // private int maxzoom = config.maxzoom();

    // private ZoomFunction<Number> labelGridPixelSize = null;
    // private ZoomFunction<Number> labelGridLimit = null;

    // private boolean attrsChangeByZoom = false;
    // private CacheByZoom<Map<String, Object>> attrCache = null;

    // private double defaultBufferPixels = 4;
    // private ZoomFunction<Number> bufferPixelOverrides;

    // // TODO better API for default value, value at max zoom, and zoom-specific overrides for tolerance and min size?
    // private double defaultMinPixelSize = config.minFeatureSizeBelowMaxZoom();
    // private double minPixelSizeAtMaxZoom = config.minFeatureSizeAtMaxZoom();
    // private ZoomFunction<Number> minPixelSize = null;

    // private double defaultPixelTolerance = config.simplifyToleranceBelowMaxZoom();
    // private double pixelToleranceAtMaxZoom = config.simplifyToleranceAtMaxZoom();
    // private ZoomFunction<Number> pixelTolerance = null;

    // private String numPointsAttr = null;
}

impl Feature {
    fn new(_layer: &str, _geom: geos::Geometry, source_id: u64) -> Feature {
        Feature {
            source_id,
            min_zoom: 10,
            max_zoom: 12,
        }
    }
    /// Returns the simplification tolerance for lines and polygons in tile pixels at `zoom`.
    fn get_pixel_tolerance_at_zoom(&self, _z: u16) -> f64 {
        // return zoom == config.maxzoom() ? pixelToleranceAtMaxZoom
        //   : ZoomFunction.applyAsDoubleOrElse(pixelTolerance, zoom, defaultPixelTolerance);
        256.0 / 4096.0
    }
    /// Returns the minimum resolution in tile pixels of features to emit at {@code zoom}.
    ///
    /// For line features, this is length, and for polygon features this is the square root of the minimum area of
    /// features to emit.
    fn get_min_pixel_size_at_zoom(&self, _z: u16) -> f64 {
        // return zoom == config.maxzoom() ? minPixelSizeAtMaxZoom
        //   : ZoomFunction.applyAsDoubleOrElse(minPixelSize, zoom, defaultMinPixelSize);
        1.
    }
    /// Returns the number of pixels of detail to render outside the visible tile boundary at {@code zoom}.
    fn get_buffer_pixels_at_zoom(&self, _z: u16) -> f64 {
        // return ZoomFunction.applyAsDoubleOrElse(bufferPixelOverrides, zoom, defaultBufferPixels);
        4.0
    }
    fn get_attrs_at_zoom(&self, _z: u16) -> HashMap<String, String> {
        HashMap::new() // Map<String, Object>
    }
}

pub fn render_line_or_polygon(feature: Feature, input: geos::Geometry) {
    let id: u64 = 0; // idGenerator.incrementAndGet();
    let area = false;
    let world_length = 0.0; // (area || input.getNumGeometries() > 1) ? 0 : input.getLength();
    let num_points_attr: Option<String> = None; // feature.getNumPointsAttr();
    for z in feature.max_zoom..=feature.min_zoom {
        let scale = (1 << z) as f64;
        let tolerance = feature.get_pixel_tolerance_at_zoom(z) / 256.0;
        let mut min_size = feature.get_min_pixel_size_at_zoom(z) / 256.0;
        if area {
            // treat minPixelSize as the edge of a square that defines minimum area for features
            min_size *= min_size;
        } else if world_length > 0.0 && world_length * scale < min_size {
            // skip linestring, too short
            continue;
        }

        let geom = scaled_geom(&input, scale);
        // Douglas/Peucker
        let geom = geom
            .topology_preserve_simplify(tolerance)
            .expect("topology_preserve_simplify");

        let groups = extract_groups(geom, min_size).expect("extract_groups");
        let buffer = feature.get_buffer_pixels_at_zoom(z) / 256.0;
        let extents = vec![0]; // TileExtents.ForZoom extents = config.bounds().tileExtents().getForZoom(z);
        let sliced =
            TiledGeometry::slice_into_tiles(groups, buffer, area, z, extents, feature.source_id);
        let attrs: HashMap<String, String> = feature.get_attrs_at_zoom(z); // get_attrs_at_zoom(sliced.zoomLevel())
                                                                           // if (numPointsAttr != null) {
                                                                           //   // if profile wants the original number of points that the simplified but untiled geometry started with
                                                                           //   attrs = new HashMap<>(attrs);
                                                                           //   attrs.put(numPointsAttr, geom.getNumPoints());
                                                                           // }
        write_tile_features(z, id, &feature, sliced, attrs);
    }
}

fn write_tile_features(
    z: u16,
    id: u64,
    feature: &Feature,
    sliced: TiledGeometry,
    attrs: HashMap<String, String>,
) {
}

fn scaled_geom<'a, 'b>(input: &'a geos::Geometry<'b>, scale: f64) -> geos::Geometry<'b>
where
    'b: 'a,
{
    Clone::clone(&input)
}

struct TiledGeometry<'a>(Vec<geos::Geometry<'a>>);

impl<'a> TiledGeometry<'a> {
    fn slice_into_tiles(
        groups: Vec<Vec<CoordSeq>>,
        buffer: f64,
        area: bool,
        z: u16,
        extents: Vec<u8>,
        source_id: u64,
    ) -> TiledGeometry<'a> {
        todo!()
    }
}

// private void writeTileFeatures(int zoom, long id, FeatureCollector.Feature feature, TiledGeometry sliced,
//   Map<String, Object> attrs) {
//   int emitted = 0;
//   for (var entry : sliced.getTileData()) {
//     TileCoord tile = entry.getKey();
//     try {
//       List<List<CoordinateSequence>> geoms = entry.getValue();

//       Geometry geom;
//       int scale = 0;
//       if (feature.isPolygon()) {
//         geom = GeometryCoordinateSequences.reassemblePolygons(geoms);
//         /*
//          * Use the very expensive, but necessary JTS Geometry#buffer(0) trick to repair invalid polygons (with self-
//          * intersections) and JTS GeometryPrecisionReducer utility to snap polygon nodes to the vector tile grid
//          * without introducing self-intersections.
//          *
//          * See https://docs.mapbox.com/vector-tiles/specification/#simplification for issues that can arise from naive
//          * coordinate rounding.
//          */
//         geom = GeoUtils.snapAndFixPolygon(geom);
//         // JTS utilities "fix" the geometry to be clockwise outer/CCW inner but vector tiles flip Y coordinate,
//         // so we need outer CCW/inner clockwise
//         geom = geom.reverse();
//       } else {
//         geom = GeometryCoordinateSequences.reassembleLineStrings(geoms);
//         // Store lines with extra precision (2^scale) in intermediate feature storage so that
//         // rounding does not introduce artificial endpoint intersections and confuse line merge
//         // post-processing.  Features need to be "unscaled" in FeatureGroup after line merging,
//         // and before emitting to output mbtiles.
//         scale = Math.max(config.maxzoom(), 14) - zoom;
//         // need 14 bits to represent tile coordinates (4096 * 2 for buffer * 2 for zig zag encoding)
//         // so cap the scale factor to avoid overflowing 32-bit integer space
//         scale = Math.min(31 - 14, scale);
//       }

//       if (!geom.isEmpty()) {
//         encodeAndEmitFeature(feature, id, attrs, tile, geom, null, scale);
//         emitted++;
//       }
//     } catch (GeometryException e) {
//       e.log(stats, "write_tile_features", "Error writing tile " + tile + " feature " + feature);
//     }
//   }

//   // polygons that span multiple tiles contain detail about the outer edges separate from the filled tiles, so emit
//   // filled tiles now
//   if (feature.isPolygon()) {
//     emitted += emitFilledTiles(id, feature, sliced);
//   }

//   stats.emittedFeatures(zoom, feature.getLayer(), emitted);
// }

fn extract_groups(geom: geos::Geometry, min_size: f64) -> GResult<Vec<Vec<CoordSeq>>> {
    let mut groups = Vec::new();
    let _type = geom.geometry_type();
    match _type {
        GeometryTypes::LineString | GeometryTypes::LinearRing => {
            let cs = geom.get_coord_seq()?;
            groups.push(vec![cs]);
        }
        GeometryTypes::MultiLineString => {
            let n_lines = geom.get_num_geometries()?;
            let mut result_lines = Vec::with_capacity(n_lines);
            for i in 0..n_lines {
                let cs = geom.get_geometry_n(i)?.get_coord_seq()?;
                result_lines.push(cs);
            }
            groups.push(result_lines);
        }
        GeometryTypes::Polygon => {
            let nb_interiors = geom.get_num_interior_rings()?;
            let mut rings = Vec::with_capacity(nb_interiors + 1);
            // Exterior ring to coordinates
            rings.push(geom.get_exterior_ring()?.get_coord_seq()?);
            // Interior rings to coordinates
            for ix_interior in 0..nb_interiors {
                rings.push(
                    geom.get_interior_ring_n(ix_interior as u32)?
                        .get_coord_seq()?,
                );
            }
            groups.push(rings);
        }
        GeometryTypes::MultiPolygon => {
            let n_polygs = geom.get_num_geometries()?;
            // let mut result_polygs = Vec::with_capacity(n_polygs);
            for i in 0..n_polygs {
                let polyg = geom.get_geometry_n(i)?;
                let nb_interiors = polyg.get_num_interior_rings()?;

                let mut rings = Vec::with_capacity(nb_interiors + 1);
                // Exterior ring to coordinates
                rings.push(polyg.get_exterior_ring()?.get_coord_seq()?);
                // Interior rings to coordinates
                for ix_interior in 0..nb_interiors {
                    rings.push(
                        polyg
                            .get_interior_ring_n(ix_interior as u32)?
                            .get_coord_seq()?,
                    );
                }
                // result_polygs.push(rings);
                groups.push(rings);
            }
        }
        GeometryTypes::GeometryCollection => {
            let n_geoms = geom.get_num_geometries()?;
            // let mut result_geoms = Vec::with_capacity(n_geoms);
            for i in 0..n_geoms {
                let g = geom.get_geometry_n(i)?;
            }
        }
        GeometryTypes::Point | GeometryTypes::MultiPoint => {
            return Err(geos::Error::GenericError(
                "unexpected geometry type".to_string(),
            ));
        }
        _ => unreachable!(),
    };
    Ok(groups)
}

#[test]
fn test_point() {
    assert!(true)
}

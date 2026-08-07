//! Persistent spatial-resolution coverage for frontier materialization.
//!
//! Coverage cells and chunks are an internal server index, not a player-facing
//! map. A resolved bit means stellar existence has been decided for the whole
//! cell; it does not mean any particular player knows the result.

use std::collections::BTreeMap;

use thiserror::Error;

pub const COVERAGE_CELL_EDGE_PARSECS: f64 = 0.25;
pub const COVERAGE_CELLS_PER_CHUNK_AXIS: i64 = 32;
pub const COVERAGE_CHUNK_EDGE_PARSECS: f64 =
    COVERAGE_CELL_EDGE_PARSECS * COVERAGE_CELLS_PER_CHUNK_AXIS as f64;
pub const COVERAGE_CELLS_PER_CHUNK: usize = 32 * 32 * 32;
pub const COVERAGE_BITMAP_BYTES: usize = COVERAGE_CELLS_PER_CHUNK / 8;
pub const JUMP_ARRIVAL_MAPPING_RADIUS_PARSECS: f64 = 6.0;
pub const COVERAGE_SAMPLER_VERSION: u16 = 1;

const CELLS_PER_PARSEC: f64 = 1.0 / COVERAGE_CELL_EDGE_PARSECS;
const MAX_FOOTPRINT_RADIUS_PARSECS: f64 = JUMP_ARRIVAL_MAPPING_RADIUS_PARSECS;
const MAX_SETTLEMENT_FOOTPRINT_RADIUS_PARSECS: f64 = 512.0;
const MAX_ABSOLUTE_CELL_COORDINATE: f64 = 1_000_000_000.0;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoverageChunkCoordinate {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellBitmap {
    bytes: Box<[u8; COVERAGE_BITMAP_BYTES]>,
}

impl CellBitmap {
    pub fn empty() -> Self {
        Self {
            bytes: Box::new([0; COVERAGE_BITMAP_BYTES]),
        }
    }

    pub fn from_bytes(bytes: [u8; COVERAGE_BITMAP_BYTES]) -> Self {
        Self {
            bytes: Box::new(bytes),
        }
    }

    pub fn filled() -> Self {
        Self {
            bytes: Box::new([u8::MAX; COVERAGE_BITMAP_BYTES]),
        }
    }

    pub fn as_bytes(&self) -> &[u8; COVERAGE_BITMAP_BYTES] {
        &self.bytes
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.iter().all(|byte| *byte == 0)
    }

    pub fn count_ones(&self) -> u64 {
        self.bytes
            .iter()
            .map(|byte| u64::from(byte.count_ones()))
            .sum()
    }

    pub fn contains(&self, bit_index: usize) -> bool {
        assert!(bit_index < COVERAGE_CELLS_PER_CHUNK);
        self.bytes[bit_index / 8] & (1 << (bit_index % 8)) != 0
    }

    pub fn set(&mut self, bit_index: usize) {
        assert!(bit_index < COVERAGE_CELLS_PER_CHUNK);
        self.bytes[bit_index / 8] |= 1 << (bit_index % 8);
    }

    pub fn clear(&mut self, bit_index: usize) {
        assert!(bit_index < COVERAGE_CELLS_PER_CHUNK);
        self.bytes[bit_index / 8] &= !(1 << (bit_index % 8));
    }

    pub fn union_with(&mut self, other: &Self) {
        for (left, right) in self.bytes.iter_mut().zip(other.bytes.iter()) {
            *left |= *right;
        }
    }

    pub fn subtract(&mut self, other: &Self) {
        for (left, right) in self.bytes.iter_mut().zip(other.bytes.iter()) {
            *left &= !right;
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.bytes
            .iter()
            .zip(other.bytes.iter())
            .any(|(left, right)| left & right != 0)
    }
}

impl Default for CellBitmap {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageLayer {
    pub stellar_distribution_version: u16,
    pub sampler_version: u16,
    pub resolved_cells: CellBitmap,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoverageChunk {
    pub layers: Vec<CoverageLayer>,
}

impl CoverageChunk {
    pub fn resolved_cells(&self) -> CellBitmap {
        let mut result = CellBitmap::empty();
        for layer in &self.layers {
            result.union_with(&layer.resolved_cells);
        }
        result
    }

    /// Add only cells that have not been resolved by an earlier layer.
    pub fn add_resolution(
        &mut self,
        stellar_distribution_version: u16,
        sampler_version: u16,
        requested: &CellBitmap,
    ) -> u64 {
        let mut fresh = requested.clone();
        fresh.subtract(&self.resolved_cells());
        let added = fresh.count_ones();
        if added == 0 {
            return 0;
        }
        if let Some(layer) = self.layers.iter_mut().find(|layer| {
            layer.stellar_distribution_version == stellar_distribution_version
                && layer.sampler_version == sampler_version
        }) {
            layer.resolved_cells.union_with(&fresh);
        } else {
            self.layers.push(CoverageLayer {
                stellar_distribution_version,
                sampler_version,
                resolved_cells: fresh,
            });
            self.layers
                .sort_by_key(|layer| (layer.stellar_distribution_version, layer.sampler_version));
        }
        added
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MappingCoverage {
    FullyMapped {
        coverage_revision: u64,
        footprint_cells: u64,
    },
    NeedsMaterialization {
        coverage_revision: u64,
        footprint_cells: u64,
        missing_cells: BTreeMap<CoverageChunkCoordinate, CellBitmap>,
    },
}

impl MappingCoverage {
    pub fn coverage_revision(&self) -> u64 {
        match self {
            Self::FullyMapped {
                coverage_revision, ..
            }
            | Self::NeedsMaterialization {
                coverage_revision, ..
            } => *coverage_revision,
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CoverageGeometryError {
    #[error("coverage center coordinates must be finite")]
    NonFiniteCenter,
    #[error("coverage radius must be finite and in (0, 6] parsecs")]
    InvalidRadius,
    #[error("coverage coordinates are outside the supported Galactic range")]
    CoordinateOutOfRange,
}

fn scaled_cell_floor(value: f64) -> Result<i64, CoverageGeometryError> {
    let scaled = (value * CELLS_PER_PARSEC).floor();
    if scaled.abs() > MAX_ABSOLUTE_CELL_COORDINATE {
        return Err(CoverageGeometryError::CoordinateOutOfRange);
    }
    Ok(scaled as i64)
}

fn scaled_cell_upper(value: f64) -> Result<i64, CoverageGeometryError> {
    let scaled = (value * CELLS_PER_PARSEC).ceil() - 1.0;
    if scaled.abs() > MAX_ABSOLUTE_CELL_COORDINATE {
        return Err(CoverageGeometryError::CoordinateOutOfRange);
    }
    Ok(scaled as i64)
}

fn chunk_and_local(cell_x: i64, cell_y: i64, cell_z: i64) -> (CoverageChunkCoordinate, usize) {
    let chunk = CoverageChunkCoordinate {
        x: cell_x.div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
        y: cell_y.div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
        z: cell_z.div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
    };
    let local_x = cell_x.rem_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS) as usize;
    let local_y = cell_y.rem_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS) as usize;
    let local_z = cell_z.rem_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS) as usize;
    let bit_index = (local_x * 32 + local_y) * 32 + local_z;
    (chunk, bit_index)
}

/// Return the persistent chunk and bit containing a finite point.
///
/// Coverage cells are half-open on their positive faces, matching the floor
/// convention used by sphere footprints.
pub fn point_cell(
    position_parsecs: [f64; 3],
) -> Result<(CoverageChunkCoordinate, usize), CoverageGeometryError> {
    if !position_parsecs
        .iter()
        .all(|coordinate| coordinate.is_finite())
    {
        return Err(CoverageGeometryError::NonFiniteCenter);
    }
    let cells = [
        scaled_cell_floor(position_parsecs[0])?,
        scaled_cell_floor(position_parsecs[1])?,
        scaled_cell_floor(position_parsecs[2])?,
    ];
    Ok(chunk_and_local(cells[0], cells[1], cells[2]))
}

fn axis_distance_to_cell(coordinate: f64, cell_coordinate: i64) -> f64 {
    let minimum = cell_coordinate as f64 * COVERAGE_CELL_EDGE_PARSECS;
    let maximum = minimum + COVERAGE_CELL_EDGE_PARSECS;
    if coordinate < minimum {
        minimum - coordinate
    } else if coordinate > maximum {
        coordinate - maximum
    } else {
        0.0
    }
}

/// Return every resolution cell with positive-volume intersection with a
/// sphere. Tangency-only cells are excluded because boundaries have zero
/// probability under the materialization process.
pub fn sphere_footprint_masks(
    center_parsecs: [f64; 3],
    radius_parsecs: f64,
) -> Result<BTreeMap<CoverageChunkCoordinate, CellBitmap>, CoverageGeometryError> {
    if !center_parsecs
        .iter()
        .all(|coordinate| coordinate.is_finite())
    {
        return Err(CoverageGeometryError::NonFiniteCenter);
    }
    if !radius_parsecs.is_finite()
        || radius_parsecs <= 0.0
        || radius_parsecs > MAX_FOOTPRINT_RADIUS_PARSECS
    {
        return Err(CoverageGeometryError::InvalidRadius);
    }

    let minimum = [
        scaled_cell_floor(center_parsecs[0] - radius_parsecs)?,
        scaled_cell_floor(center_parsecs[1] - radius_parsecs)?,
        scaled_cell_floor(center_parsecs[2] - radius_parsecs)?,
    ];
    let maximum = [
        scaled_cell_upper(center_parsecs[0] + radius_parsecs)?,
        scaled_cell_upper(center_parsecs[1] + radius_parsecs)?,
        scaled_cell_upper(center_parsecs[2] + radius_parsecs)?,
    ];
    let radius_squared = radius_parsecs * radius_parsecs;
    let mut chunks = BTreeMap::<CoverageChunkCoordinate, CellBitmap>::new();
    for cell_x in minimum[0]..=maximum[0] {
        let dx = axis_distance_to_cell(center_parsecs[0], cell_x);
        for cell_y in minimum[1]..=maximum[1] {
            let dy = axis_distance_to_cell(center_parsecs[1], cell_y);
            for cell_z in minimum[2]..=maximum[2] {
                let dz = axis_distance_to_cell(center_parsecs[2], cell_z);
                if dx * dx + dy * dy + dz * dz >= radius_squared {
                    continue;
                }
                let (chunk, bit_index) = chunk_and_local(cell_x, cell_y, cell_z);
                chunks.entry(chunk).or_default().set(bit_index);
            }
        }
    }
    Ok(chunks)
}

/// Return a large one-time settlement-envelope footprint.
///
/// Ordinary Jump materialization remains capped at six parsecs. This separate
/// operation exists for an explicitly requested, bounded bulk survey such as
/// the capacity fixture. Whole chunks strictly inside the sphere are filled
/// without visiting their 32,768 individual cells; only boundary chunks need
/// the exact positive-volume cell test.
pub fn settlement_sphere_footprint_masks(
    center_parsecs: [f64; 3],
    radius_parsecs: f64,
) -> Result<BTreeMap<CoverageChunkCoordinate, CellBitmap>, CoverageGeometryError> {
    if !center_parsecs
        .iter()
        .all(|coordinate| coordinate.is_finite())
    {
        return Err(CoverageGeometryError::NonFiniteCenter);
    }
    if !radius_parsecs.is_finite()
        || radius_parsecs <= 0.0
        || radius_parsecs > MAX_SETTLEMENT_FOOTPRINT_RADIUS_PARSECS
    {
        return Err(CoverageGeometryError::InvalidRadius);
    }

    let minimum_cell = [
        scaled_cell_floor(center_parsecs[0] - radius_parsecs)?,
        scaled_cell_floor(center_parsecs[1] - radius_parsecs)?,
        scaled_cell_floor(center_parsecs[2] - radius_parsecs)?,
    ];
    let maximum_cell = [
        scaled_cell_upper(center_parsecs[0] + radius_parsecs)?,
        scaled_cell_upper(center_parsecs[1] + radius_parsecs)?,
        scaled_cell_upper(center_parsecs[2] + radius_parsecs)?,
    ];
    let minimum_chunk = [
        minimum_cell[0].div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
        minimum_cell[1].div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
        minimum_cell[2].div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
    ];
    let maximum_chunk = [
        maximum_cell[0].div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
        maximum_cell[1].div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
        maximum_cell[2].div_euclid(COVERAGE_CELLS_PER_CHUNK_AXIS),
    ];
    let radius_squared = radius_parsecs * radius_parsecs;
    let mut chunks = BTreeMap::new();
    for chunk_x in minimum_chunk[0]..=maximum_chunk[0] {
        for chunk_y in minimum_chunk[1]..=maximum_chunk[1] {
            for chunk_z in minimum_chunk[2]..=maximum_chunk[2] {
                let chunk = CoverageChunkCoordinate {
                    x: chunk_x,
                    y: chunk_y,
                    z: chunk_z,
                };
                let chunk_minimum = [
                    chunk_x as f64 * COVERAGE_CHUNK_EDGE_PARSECS,
                    chunk_y as f64 * COVERAGE_CHUNK_EDGE_PARSECS,
                    chunk_z as f64 * COVERAGE_CHUNK_EDGE_PARSECS,
                ];
                let chunk_maximum = [
                    chunk_minimum[0] + COVERAGE_CHUNK_EDGE_PARSECS,
                    chunk_minimum[1] + COVERAGE_CHUNK_EDGE_PARSECS,
                    chunk_minimum[2] + COVERAGE_CHUNK_EDGE_PARSECS,
                ];
                let mut minimum_distance_squared = 0.0;
                let mut maximum_distance_squared = 0.0;
                for axis in 0..3 {
                    let minimum_distance = if center_parsecs[axis] < chunk_minimum[axis] {
                        chunk_minimum[axis] - center_parsecs[axis]
                    } else if center_parsecs[axis] > chunk_maximum[axis] {
                        center_parsecs[axis] - chunk_maximum[axis]
                    } else {
                        0.0
                    };
                    minimum_distance_squared += minimum_distance * minimum_distance;
                    let farthest = (center_parsecs[axis] - chunk_minimum[axis])
                        .abs()
                        .max((center_parsecs[axis] - chunk_maximum[axis]).abs());
                    maximum_distance_squared += farthest * farthest;
                }
                if minimum_distance_squared >= radius_squared {
                    continue;
                }
                if maximum_distance_squared <= radius_squared {
                    chunks.insert(chunk, CellBitmap::filled());
                    continue;
                }

                let mut cells = CellBitmap::empty();
                for local_x in 0..COVERAGE_CELLS_PER_CHUNK_AXIS {
                    let cell_x = chunk_x * COVERAGE_CELLS_PER_CHUNK_AXIS + local_x;
                    let dx = axis_distance_to_cell(center_parsecs[0], cell_x);
                    for local_y in 0..COVERAGE_CELLS_PER_CHUNK_AXIS {
                        let cell_y = chunk_y * COVERAGE_CELLS_PER_CHUNK_AXIS + local_y;
                        let dy = axis_distance_to_cell(center_parsecs[1], cell_y);
                        for local_z in 0..COVERAGE_CELLS_PER_CHUNK_AXIS {
                            let cell_z = chunk_z * COVERAGE_CELLS_PER_CHUNK_AXIS + local_z;
                            let dz = axis_distance_to_cell(center_parsecs[2], cell_z);
                            if dx * dx + dy * dy + dz * dz >= radius_squared {
                                continue;
                            }
                            let (_, bit_index) = chunk_and_local(cell_x, cell_y, cell_z);
                            cells.set(bit_index);
                        }
                    }
                }
                if !cells.is_empty() {
                    chunks.insert(chunk, cells);
                }
            }
        }
    }
    Ok(chunks)
}

pub fn jump_arrival_footprint_masks(
    target_parsecs: [f64; 3],
) -> Result<BTreeMap<CoverageChunkCoordinate, CellBitmap>, CoverageGeometryError> {
    sphere_footprint_masks(target_parsecs, JUMP_ARRIVAL_MAPPING_RADIUS_PARSECS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell_count(chunks: &BTreeMap<CoverageChunkCoordinate, CellBitmap>) -> u64 {
        chunks.values().map(CellBitmap::count_ones).sum()
    }

    #[test]
    fn jump_footprint_is_stable_across_negative_chunk_boundaries() {
        let origin = jump_arrival_footprint_masks([0.0; 3]).unwrap();
        let shifted = jump_arrival_footprint_masks([8.0, -8.0, 16.0]).unwrap();
        assert_eq!(cell_count(&origin), cell_count(&shifted));
        assert_eq!(cell_count(&origin), 63_256);
        assert_eq!(origin.len(), 8);
        assert_eq!(shifted.len(), 8);
        for coordinate in origin.keys() {
            let translated = CoverageChunkCoordinate {
                x: coordinate.x + 1,
                y: coordinate.y - 1,
                z: coordinate.z + 2,
            };
            assert_eq!(origin[coordinate], shifted[&translated]);
        }
    }

    #[test]
    fn a_quarter_parsec_shift_changes_the_footprint_deterministically() {
        let first = jump_arrival_footprint_masks([1.0, -2.0, 0.5]).unwrap();
        let second = jump_arrival_footprint_masks([1.25, -2.0, 0.5]).unwrap();
        assert_eq!(cell_count(&first), cell_count(&second));
        assert_ne!(first, second);
    }

    #[test]
    fn settlement_footprint_matches_the_jump_algorithm_at_small_radius() {
        let ordinary = sphere_footprint_masks([1.25, -2.0, 0.5], 6.0).unwrap();
        let settlement = settlement_sphere_footprint_masks([1.25, -2.0, 0.5], 6.0).unwrap();
        assert_eq!(settlement, ordinary);
    }

    #[test]
    #[ignore = "capacity-scale benchmark; run explicitly with --ignored"]
    fn settlement_footprint_supports_the_capacity_envelope() {
        let footprint = settlement_sphere_footprint_masks([0.0; 3], 90.0).unwrap();
        let cells = cell_count(&footprint);
        let represented_volume = cells as f64 * COVERAGE_CELL_EDGE_PARSECS.powi(3);
        let sphere_volume = 4.0 * std::f64::consts::PI * 90.0_f64.powi(3) / 3.0;
        // Boundary cells are intentionally included when they have any
        // positive-volume overlap, so the rasterized volume is a close upper
        // approximation rather than an exact analytic sphere.
        assert!(represented_volume >= sphere_volume);
        assert!(represented_volume < sphere_volume * 1.01);
    }

    #[test]
    fn chunk_layers_never_reassign_resolved_cells() {
        let mut first = CellBitmap::empty();
        first.set(17);
        first.set(900);
        let mut second = CellBitmap::empty();
        second.set(900);
        second.set(901);
        let mut chunk = CoverageChunk::default();
        assert_eq!(chunk.add_resolution(1, 1, &first), 2);
        assert_eq!(chunk.add_resolution(2, 1, &second), 1);
        assert_eq!(chunk.layers.len(), 2);
        assert_eq!(chunk.layers[0].resolved_cells.count_ones(), 2);
        assert_eq!(chunk.layers[1].resolved_cells.count_ones(), 1);
        assert!(chunk.resolved_cells().contains(901));
    }

    #[test]
    fn invalid_footprints_are_rejected_before_iteration() {
        assert_eq!(
            jump_arrival_footprint_masks([f64::NAN, 0.0, 0.0]),
            Err(CoverageGeometryError::NonFiniteCenter)
        );
        assert_eq!(
            sphere_footprint_masks([0.0; 3], 6.25),
            Err(CoverageGeometryError::InvalidRadius)
        );
    }

    #[test]
    fn point_cells_use_half_open_quarter_parsec_boundaries() {
        let (negative_chunk, negative_bit) = point_cell([-0.001, 0.0, 0.0]).unwrap();
        let (origin_chunk, origin_bit) = point_cell([0.0, 0.0, 0.0]).unwrap();
        let (next_chunk, next_bit) = point_cell([0.25, 0.0, 0.0]).unwrap();
        assert_eq!(
            negative_chunk,
            CoverageChunkCoordinate { x: -1, y: 0, z: 0 }
        );
        assert_eq!(origin_chunk, CoverageChunkCoordinate { x: 0, y: 0, z: 0 });
        assert_eq!(next_chunk, origin_chunk);
        assert_ne!(negative_bit, origin_bit);
        assert_ne!(origin_bit, next_bit);
    }
}

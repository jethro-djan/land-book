use rusqlite::{Connection, Result, params};
use flate2::read::GzDecoder;
use std::io::Read;
use prost::Message;

use crate::vector_tile::Tile;
use crate::vector_tile::tile::{Feature, GeomType};

const MOVE_TO: u32 = 1;
const LINE_TO: u32 = 2;
const CLOSE_PATH: u32 = 7;

pub struct TileWithoutData {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

pub struct TileWithData {
    pub z: u32,
    pub x: u32,
    pub y: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GeometryType {
    Point(Vec<(i32, i32)>),
    LineString(Vec<Vec<(i32, i32)>>),
    Polygon(Vec<Vec<(i32, i32)>>),
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeatureCommand {
    MoveTo = MOVE_TO,
    LineTo = LINE_TO,
    ClosePath = CLOSE_PATH,
}

impl TryFrom<u32> for FeatureCommand {
    type Error = ();

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            MOVE_TO => Ok(Self::MoveTo),
            LINE_TO => Ok(Self::LineTo),
            CLOSE_PATH => Ok(Self::ClosePath),
            _ => Err(())
        }
    }
}

pub struct MbTiles {
    conn: Connection,
}

impl MbTiles {
    pub fn open_local(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn get_tile(&self, tile: TileWithoutData) -> Result<TileWithData> {
        // Flip y from slippy map convention to TMS convention
        let tms_y = (1u32 << tile.z).wrapping_sub(1).wrapping_sub(tile.y);

        let mut stmt = self.conn.prepare_cached(
            r#"SELECT tile_data FROM tiles
                WHERE zoom_level = ?1
                AND tile_column = ?2
                AND tile_row = ?3
            "#
        )?;

        let data = stmt.query_row(params![tile.z, tile.x, tms_y], |row| {
            row.get::<_, Vec<u8>>(0)
        })?;

        Ok(TileWithData {
            z: tile.z,
            x: tile.x,
            y: tms_y,
            data,
        })
    }

    pub fn decode_tile_with_data(tile: TileWithData) -> Tile {
        let mut decoder = GzDecoder::new(tile.data.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        Tile::decode(decompressed.as_slice()).unwrap()
    }
}

fn zigzag_decode(n: u32) -> i32 {
    ((n >> 1) as i32) ^ -((n & 1) as i32)
}


pub fn decode_geometry(feature: &Feature) -> Option<GeometryType> {
    let geom = &feature.geometry;
    let mut i = 0;
    let mut cursor_x: i32 = 0;
    let mut cursor_y: i32 = 0;

    match feature.r#type() {
        GeomType::Point => {
            let mut points = Vec::new();

            while i < geom.len() {
                let command_id = geom[i] & 0x7;
                let count = geom[i] >> 3;
                i += 1;

                let command = FeatureCommand::try_from(command_id).ok()?;

                match command {
                    FeatureCommand::MoveTo => {
                        for _ in 0..count {
                            let dx = zigzag_decode(geom[i]);
                            let dy = zigzag_decode(geom[i + 1]);
                            i += 2;

                            cursor_x += dx;
                            cursor_y += dy;
                            points.push((cursor_x, cursor_y));
                        }
                    }
                    FeatureCommand::LineTo | FeatureCommand::ClosePath => return None
                }
            }

            Some(GeometryType::Point(points))
        }
        GeomType::Linestring => {
            let mut lines = Vec::new();

            while i < geom.len() {
                let command_id = geom[i] & 0x7;
                let count = geom[i] >> 3;
                i += 1;

                let command = FeatureCommand::try_from(command_id).ok()?;

                match command {
                    FeatureCommand::MoveTo => {
                        let dx = zigzag_decode(geom[i]);
                        let dy = zigzag_decode(geom[i + 1]);
                        i += 2;

                        cursor_x += dx;
                        cursor_y += dy;

                        lines.push(vec![(cursor_x, cursor_y)]);
                    }
                    FeatureCommand::LineTo => {
                        let current_line = lines.last_mut().unwrap();

                        for _ in 0..count {
                            let dx = zigzag_decode(geom[i]);
                            let dy = zigzag_decode(geom[i + 1]);
                            i += 2;
                                
                            cursor_x += dx;
                            cursor_y += dy;
                            current_line.push((cursor_x, cursor_y));
                        }
                    }
                    FeatureCommand::ClosePath => return None
                }
            }

            Some(GeometryType::LineString(lines))
        }
        GeomType::Polygon => {
            let mut rings = Vec::new();

            while i < geom.len() {
                let command_id = geom[i] & 0x7;
                let count = geom[i] >> 3;
                i += 1;

                let command = FeatureCommand::try_from(command_id).ok()?;

                match command {
                    FeatureCommand::MoveTo => {
                        let dx = zigzag_decode(geom[i]);
                        let dy = zigzag_decode(geom[i + 1]);
                        i += 2;

                        cursor_x += dx;
                        cursor_y += dy;

                        rings.push(vec![(cursor_x, cursor_y)]);
                    }
                    FeatureCommand::LineTo => {
                        let current_ring = rings.last_mut().unwrap();

                        for _ in 0..count {
                            let dx = zigzag_decode(geom[i]);
                            let dy = zigzag_decode(geom[i + 1]);
                            i += 2;

                            cursor_x += dx;
                            cursor_y += dy;
                            current_ring.push((cursor_x, cursor_y));
                        }
                    }
                    FeatureCommand::ClosePath => {
                        let current_ring = rings.last_mut().unwrap();
                        let start = current_ring[0];
                        current_ring.push(start);
                    }
                }
            }

            Some(GeometryType::Polygon(rings))
        }
        GeomType::Unknown => None
    }
}


// TESTS 

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(command_id: u32, count: u32) -> u32 {
        (count << 3) | command_id
    }

    fn zigzag_encode(n: i32) -> u32 {
        ((n << 1) ^ (n >> 31)) as u32
    }

    fn feature(kind: GeomType, geometry: Vec<u32>) -> Feature {
        Feature {
            id: 0,
            tags: Vec::new(),
            r#type: kind as i32,
            geometry,
        }
    }

    #[test]
    fn zigzag_decode_decodes_positive_and_negative_values() {
        assert_eq!(zigzag_decode(0), 0);
        assert_eq!(zigzag_decode(2), 1);
        assert_eq!(zigzag_decode(4), 2);
        assert_eq!(zigzag_decode(20), 10);

        assert_eq!(zigzag_decode(1), -1);
        assert_eq!(zigzag_decode(3), -2);
        assert_eq!(zigzag_decode(5), -3);
    }

    #[test]
    fn feature_command_accepts_valid_command_ids() {
        assert_eq!(FeatureCommand::try_from(1), Ok(FeatureCommand::MoveTo));
        assert_eq!(FeatureCommand::try_from(2), Ok(FeatureCommand::LineTo));
        assert_eq!(FeatureCommand::try_from(7), Ok(FeatureCommand::ClosePath));
    }

    #[test]
    fn feature_command_rejects_invalid_command_ids() {
        assert_eq!(FeatureCommand::try_from(0), Err(()));
        assert_eq!(FeatureCommand::try_from(3), Err(()));
        assert_eq!(FeatureCommand::try_from(8), Err(()));
    }

    #[test]
    fn decode_single_point_geometry() {
        let feature = feature(
            GeomType::Point,
            vec![
                cmd(MOVE_TO, 1),
                zigzag_encode(10),
                zigzag_encode(15),
            ],
        );

        assert_eq!(
            decode_geometry(&feature),
            Some(GeometryType::Point(vec![(10, 15)]))
        );
    }

    #[test]
    fn decode_multi_point_geometry_uses_delta_coordinates() {
        let feature = feature(
            GeomType::Point,
            vec![
                cmd(MOVE_TO, 3),
                zigzag_encode(10),
                zigzag_encode(15),
                zigzag_encode(5),
                zigzag_encode(-3),
                zigzag_encode(-2),
                zigzag_encode(8),
            ],
        );

        assert_eq!(
            decode_geometry(&feature),
            Some(GeometryType::Point(vec![
                (10, 15),
                (15, 12),
                (13, 20),
            ]))
        );
    }

    #[test]
    fn point_geometry_rejects_line_to_command() {
        let feature = feature(
            GeomType::Point,
            vec![
                cmd(LINE_TO, 1),
                zigzag_encode(10),
                zigzag_encode(15),
            ],
        );

        assert_eq!(decode_geometry(&feature), None);
    }

    #[test]
    fn point_geometry_rejects_close_path_command() {
        let feature = feature(
            GeomType::Point,
            vec![cmd(CLOSE_PATH, 1)],
        );

        assert_eq!(decode_geometry(&feature), None);
    }

    #[test]
    fn decode_linestring_geometry_uses_delta_coordinates() {
        let feature = feature(
            GeomType::Linestring,
            vec![
                cmd(MOVE_TO, 1),
                zigzag_encode(10),
                zigzag_encode(15),

                cmd(LINE_TO, 2),
                zigzag_encode(5),
                zigzag_encode(0),
                zigzag_encode(-3),
                zigzag_encode(4),
            ],
        );

        assert_eq!(
            decode_geometry(&feature),
            Some(GeometryType::LineString(vec![
                vec![
                    (10, 15),
                    (15, 15),
                    (12, 19),
                ],
            ]))
        );
    }

    #[test]
    fn linestring_geometry_rejects_close_path_command() {
        let feature = feature(
            GeomType::Linestring,
            vec![cmd(CLOSE_PATH, 1)],
        );

        assert_eq!(decode_geometry(&feature), None);
    }

    #[test]
    fn unknown_geometry_returns_none() {
        let feature = feature(GeomType::Unknown, Vec::new());

        assert_eq!(decode_geometry(&feature), None);
    }
}

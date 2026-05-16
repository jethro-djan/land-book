use rusqlite::{Connection, Result, params};
use flate2::read::GzDecoder;
use std::io::Read;
use prost::Message;

use crate::vector_tile::Tile;
use crate::vector_tile::tile::{Feature, GeomType};

const MOVE_TO: i32 = 1;
const LINE_TO: i32 = 2;
const CLOSE_PATH: i32 = 7;

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


pub fn decode_geometry(feature: &Feature) -> Option<GeomType> {
}

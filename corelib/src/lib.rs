pub mod tile_service;

pub mod corelib {
    pub mod vector_tile {
        include!(concat!(env!("OUT_DIR"), "/vector_tile.rs"));
    }
}

use corelib::vector_tile;

use super::{Map, TileType};
use rltk::{RGB, FontCharType};

pub fn tile_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg) = 
        match map.depth {
            1 | 2 | 3 => get_forest_glyph(idx, map),
            5 => get_limestone_caverns_glyph(idx, map),
            _ => get_default_glyph(idx, map)
    };

    if map.bloodstains.contains(&idx) { bg = RGB::from_f32(0.75, 0., 0.); }
    if !map.visible_tiles[idx] { 
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.); // hide out of sight bloodstains
    } else if !map.outdoors {
        fg = fg * map.light[idx];
        bg = bg * map.light[idx];
    }

    (glyph, fg, bg)
}

fn get_default_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::named(rltk::BLACK);
    
    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.0, 1.0, 0.0);
        }
        TileType::Wall => {
            let (x, y) = map.idx_xy(idx);
            glyph = wall_glyph(&*map, x, y);
            fg = RGB::from_f32(0.0, 1.0, 0.0);
        }
        TileType::TownWall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::UpStairs => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::WoodFloor => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Bridge => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        TileType::Grass => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = rltk::to_cp437('▓');
            fg = RGB::named(rltk::NAVY_BLUE);
        }
        TileType::Gravel => {
            glyph = rltk::to_cp437(';');
            fg = RGB::named(rltk::GRAY);
        }
    }

    (glyph, fg, bg)
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> FontCharType {
    if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
    let mut mask : u8 = 0;

    if is_revealed_wall(map, x, y - 1) { mask +=1; }
    if is_revealed_wall(map, x, y + 1) { mask +=2; }
    if is_revealed_wall(map, x - 1, y) { mask +=4; }
    if is_revealed_wall(map, x + 1, y) { mask +=8; }

    match mask {
        0 => { 9 } // Pillar because we can't see neighbors TODO: find a better way
        1 => { 186 } // Wall only to the north
        2 => { 186 } // Wall only to the south
        3 => { 186 } // Wall to the north and south
        4 => { 205 } // Wall only to the west
        5 => { 188 } // Wall to the north and west
        6 => { 187 } // Wall to the south and west
        7 => { 185 } // Wall to the north, south and west
        8 => { 205 } // Wall only to the east
        9 => { 200 } // Wall to the north and east
        10 => { 201 } // Wall to the south and east
        11 => { 204 } // Wall to the north, south and east
        12 => { 205 } // Wall to the east and west
        13 => { 202 } // Wall to the east, west, and south
        14 => { 203 } // Wall to the east, west, and north
        15 => { 206 }  // Wall on all sides
        _ => { 35 } // We missed one?
    }
}

fn get_forest_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::named(rltk::BLACK);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('"');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
        TileType::Wall => {
            glyph = rltk::to_cp437('♣');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
        TileType::TownWall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::UpStairs => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::WoodFloor => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Bridge => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        TileType::Grass => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = rltk::to_cp437('▓');
            fg = RGB::named(rltk::BLUE);
        }
        TileType::Gravel => {
            glyph = rltk::to_cp437(';');
            fg = RGB::named(rltk::GRAY);
        }
    }

    (glyph, fg, bg)
}

fn get_limestone_caverns_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::named(rltk::BLACK);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('░');
            fg = RGB::from_f32(0.4, 0.4, 0.4);
        }
        TileType::Wall => {
            glyph = rltk::to_cp437('▒');
            fg = RGB::from_f32(0.7, 0.7, 0.7);
        }
        TileType::TownWall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::UpStairs => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::WoodFloor => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Bridge => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        TileType::Grass => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = rltk::to_cp437('▓');
            fg = RGB::named(rltk::BLUE);
        }
        TileType::Gravel => {
            glyph = rltk::to_cp437(';');
            fg = RGB::named(rltk::GRAY);
        }
    }

    (glyph, fg, bg)
}

fn is_revealed_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}

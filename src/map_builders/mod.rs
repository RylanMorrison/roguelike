use std::collections::VecDeque;

use super::{spawner, Map, Position, Rect, TileType};
use crate::{raws::MapData, rng};
mod area_starting_points;
mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod corridors;
mod cull_unreachable;
mod distant_exit;
mod dla;
mod door_placement;
mod drunkard;
mod levels;
mod maze;
mod prefabs;
mod rooms;
mod simple_map;
mod voronoi;
mod voronoi_spawning;

use area_starting_points::{AreaStartingPosition, XStart, YStart};
use bsp_dungeon::BspDungeonBuilder;
use bsp_interior::BspInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use common::*;
use corridors::*;
use cull_unreachable::CullUnreachable;
use distant_exit::DistantExit;
use dla::DLABuilder;
use door_placement::DoorPlacement;
use drunkard::DrunkardsWalkBuilder;
use levels::*;
use maze::MazeBuilder;
use prefabs::PrefabBuilder;
use rltk::Point;
use rooms::*;
use simple_map::SimpleMapBuilder;
use voronoi::VoronoiCellBuilder;
use voronoi_spawning::VoronoiSpawning;

pub struct BuilderMap {
    pub map: Map,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub prev_maps: VecDeque<String>,
    pub next_maps: VecDeque<String>
}

impl BuilderMap {
    pub fn add_next_exit(&mut self, idx: usize) {
        let (x, y) = self.map.idx_xy(idx);
        if let Some(next_map_name) = self.next_maps.pop_front() {
            self.map.tiles[idx] = TileType::NextArea { map_name: next_map_name.clone() };
            self.map.add_transition(&next_map_name, Point::new(x, y));
        } else {
            panic!("ERROR - next map for {} not set!", self.map.name);
        }
    }

    pub fn add_next_entrance(&mut self, idx: usize) {
        let (x, y) = self.map.idx_xy(idx);
        if let Some(prev_map_name) = self.prev_maps.pop_front() {
            self.map.tiles[idx] = TileType::PreviousArea { map_name: prev_map_name.clone() };
            self.map.add_transition(&prev_map_name, Point::new(x, y));
        } else {
            panic!("ERROR - previous map for {} not set!", self.map.name);
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap);
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap
}

impl BuilderChain {
    pub fn new(map_data: &MapData) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                map: Map::new(map_data),
                rooms: None,
                corridors: None,
                history: Vec::new(),
                prev_maps: map_data.prev_maps.clone().unwrap_or(VecDeque::new()),
                next_maps: map_data.next_maps.clone().unwrap_or(VecDeque::new())
            }
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder.")
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // build the starting map
                starter.build_map(&mut self.build_data);
            }
        }

        // build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(&mut self.build_data);
        }
    }
}

pub fn level_builder(map_data: &MapData) -> BuilderChain {
    match map_data.name.as_str() {
        "Landfall" => landfall_builder(map_data),
        "Forest" => forest_builder(map_data),
        "Dark Forest" => dark_forest_builder(map_data),
        "Orc Camp" => orc_camp_builder(map_data),
        "Warboss Den" => warboss_den_builder(map_data),
        "Caverns" => caverns_builder(map_data),
        _ => random_builder(map_data)
    }
}

pub fn random_builder(map_data: &MapData) -> BuilderChain {
    let mut builder = BuilderChain::new(map_data);
    let type_roll = rng::roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(&mut builder),
        _ => random_shape_builder(&mut builder)
    }

    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());
    builder
}

fn random_room_builder(builder: &mut BuilderChain) {
    let build_roll = rng::roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new(6, 10)),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new())
    }

    if build_roll != 3 {
        // skip BSP Interior
        let sort_roll = rng::roll_dice(1, 5);
        match sort_roll {
            // randomly sort the rooms
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL))
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng::roll_dice(1, 4);
        match corridor_roll {
            // randomly pick a corridor type
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new(1))
        }

        let cspawn_roll = rng::roll_dice(1, 2);
        if cspawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng::roll_dice(1, 6);
        match modifier_roll {
            // randomly pick a room modifier (or none)
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            3 => builder.with(DLABuilder::heavy_erosion()),
            _ => {}
        }
    }

    // set the start position to the center for culling unreachable areas
    builder.with(CullUnreachable::new());

    let start_roll = rng::roll_dice(1, 2);
    match start_roll {
        // randomly pick a way to determine the player start
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position();
            builder.with(AreaStartingPosition::new(start_x, start_y, false));
        }
    }

    let exit_roll = rng::roll_dice(1, 2);
    match exit_roll {
        // randomly pick a way to determine the exit
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new())
    }

    let spawn_roll = rng::roll_dice(1, 2);
    match spawn_roll {
        // randomly pick a way to spawn entities
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new())
    }
}

fn random_shape_builder(builder: &mut BuilderChain) {
    // start with the first 5 map types and add the next one every depth
    let builder_roll = rng::roll_dice(1, 14);
    let starter: Box<dyn InitialMapBuilder>;
    match builder_roll {
        // order is important!
        1 => starter = DrunkardsWalkBuilder::open_area(),
        2 => starter = DrunkardsWalkBuilder::open_halls(),
        3 => starter = DLABuilder::walk_outwards(),
        4 => starter = DrunkardsWalkBuilder::fat_passages(),
        5 => starter = DrunkardsWalkBuilder::winding_passages(),
        6 => starter = DLABuilder::walk_inwards(),
        7 => starter = CellularAutomataBuilder::new(),
        8 => starter = DrunkardsWalkBuilder::fearful_symmetry(),
        9 => starter = DLABuilder::central_attractor(),
        10 => starter = DLABuilder::insectoid(),
        11 => starter = VoronoiCellBuilder::manhattan(),
        12 => starter = VoronoiCellBuilder::pythagoras(),
        13 => starter = VoronoiCellBuilder::chebyshev(),
        _ => starter = MazeBuilder::new()
    }
    builder.start_with(starter);

    // set the start position to the center for culling unreachable areas
    builder.with(CullUnreachable::new());

    // reset the player start to a random position
    let (start_x, start_y) = random_start_position();
    builder.with(AreaStartingPosition::new(start_x, start_y, false));

    // spawn the exit and entities
    builder.with(DistantExit::new());
    builder.with(VoronoiSpawning::new());
}

fn random_start_position() -> (XStart, YStart) {
    let x_roll = rng::roll_dice(1, 3);
    let x = if x_roll == 1 {
        XStart::LEFT
    } else if x_roll == 2 {
        XStart::RIGHT
    } else {
        XStart::CENTER
    };
    let y_roll = rng::roll_dice(1, 3);
    let y = if y_roll == 1 {
        YStart::BOTTOM
    } else if y_roll == 2 {
        YStart::TOP
    } else {
        YStart::CENTER
    };

    (x, y)
}

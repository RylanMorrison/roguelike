use super::{Map, Rect, TileType, Position, spawner, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod drunkard;
use drunkard::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod common;
use common::*;
mod dla;
use dla::DLABuilder;
mod voronoi;
use voronoi::VoronoiCellBuilder;
mod prefab_builder;
use prefab_builder::PrefabBuilder;
mod room_based_spawner;
use room_based_spawner::RoomBasedSpawner;
mod room_based_stairs;
use room_based_stairs::RoomBasedStairs;
mod room_based_starting_position;
use room_based_starting_position::RoomBasedStartingPosition;
mod area_starting_points;
use area_starting_points::{AreaStartingPosition, XStart, YStart};
mod cull_unreachable;
use cull_unreachable::CullUnreachable;
mod voronoi_spawning;
use voronoi_spawning::VoronoiSpawning;
mod distant_exit;
use distant_exit::DistantExit;
mod room_exploder;
use room_exploder::RoomExploder;
mod room_corner_rounding;
use room_corner_rounding::RoomCornerRounder;
mod corridors_dogleg;
use corridors_dogleg::DoglegCorridors;
mod corridors_bsp;
use corridors_bsp::BspCorridors;
mod rooms_corridors_nearest;
use rooms_corridors_nearest::NearestCorridors;
mod room_sorter;
use room_sorter::{RoomSorter, RoomSort};
mod room_draw;
use room_draw::RoomDrawer;
mod corridors_lines;
use corridors_lines::StraightLineCorridors;
mod corridor_spawner;
use corridor_spawner::CorridorSpawner;
mod door_placement;
use door_placement::DoorPlacement;

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap
}

impl BuilderChain {
    pub fn new(new_depth: i32, width: i32, height: i32) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height
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

    pub fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entites(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height);
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(new_depth, rng, &mut builder)
    }

    if new_depth >= 10 && rng.roll_dice(1, 3) == 1 {
        // only have a chance to add a fort from depth 10 onwards
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }
    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());
    builder
}

fn random_room_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new())
    }

    if build_roll != 3 { // skip BSP Interior
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            // randomly sort the rooms
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL))
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng.roll_dice(1, 4);
        match corridor_roll {
            // randomly pick a corridor type
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new())
        }

        let cspawn_roll = rng.roll_dice(1, 2);
        if cspawn_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            // randomly pick a room modifier (or none)
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            3 => builder.with(DLABuilder::heavy_erosion()),
            _ => {}
        }
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        // randomly pick a way to determine the player start
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y));
        }
    }

    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        // randomly pick a way to determine the exit
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new())
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        // randomly pick a way to spawn entities
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new())
    }
}

fn random_shape_builder(new_depth: i32, rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    // start with the first 5 map types and add the next one very depth
    let builder_roll = rng.roll_dice(1, new_depth + 4); 
    let starter: Box<dyn InitialMapBuilder>;
    match builder_roll { // order is important!
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

    // set the player start to the center and cull unreachable areas
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    // reset the player start the a random position
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    // spawn the exit and entities
    builder.with(DistantExit::new());
    builder.with(VoronoiSpawning::new());
}

fn random_start_position(rng: &mut RandomNumberGenerator) -> (XStart, YStart) {
    let x_roll = rng.roll_dice(1, 3);
    let x = if x_roll == 1 {
        XStart::LEFT
    } else if x_roll == 2 {
        XStart::RIGHT
    } else {
        XStart::CENTER
    };
    let y_roll = rng.roll_dice(1, 3);
    let y = if y_roll == 1 {
        YStart::BOTTOM
    } else if y_roll == 2 {
        YStart::TOP
    } else {
        YStart::CENTER
    };

    (x,y)
}

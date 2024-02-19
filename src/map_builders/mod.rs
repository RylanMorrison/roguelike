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
use voronoi::VoronoiBuilder;
mod prefab_builder;
use prefab_builder::PrefabBuilder;
use rltk::console;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn take_snapshot(&mut self);
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;

    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

/// Returns a semi-random map builder depending on the map depth
pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::new();

    let mut builder = rng.roll_dice(1, new_depth + 6); // include the first 7 map types by default
    builder += depth_modifier(new_depth); // remove lower maps at higher depths

    let mut result: Box<dyn MapBuilder>;
    match builder { // order is important!
        1 => { result = Box::new(SimpleMapBuilder::new(new_depth)) },
        2 => { result = Box::new(BspInteriorBuilder::new(new_depth)) },
        3 => { result = Box::new(BspDungeonBuilder::new(new_depth)) },
        4 => { result = Box::new(CellularAutomataBuilder::new(new_depth)) },
        5 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)) },
        6 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)) },
        7 => { result = Box::new(DLABuilder::walk_outwards(new_depth)) },
        8 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)) },
        9 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)) },
        10 => { result = Box::new(DLABuilder::walk_inwards(new_depth)) },
        11 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)) },
        12 => { result = Box::new(DLABuilder::central_attractor(new_depth)) },
        13 => { result = Box::new(DLABuilder::insectoid(new_depth)) },
        14 => { result = Box::new(VoronoiBuilder::manhattan(new_depth)) },
        15 => { result = Box::new(VoronoiBuilder::pythagoras(new_depth)) },
        16 => { result = Box::new(VoronoiBuilder::chebyshev(new_depth)) },
        _ => { result = Box::new(MazeBuilder::new(new_depth)) }
    }

    if new_depth > 10 && rng.roll_dice(1, 5) == 1 {
        result = Box::new(PrefabBuilder::new_section(new_depth, prefab_builder::prefab_sections::UNDERGROUND_FORT, result));
    }  
    result = Box::new(PrefabBuilder::new_room(new_depth, result));
    result
}

fn depth_modifier(depth: i32) -> i32 {
    if depth < 6 {
        0
    } else if depth < 10 {
        3
    } else {
        6
    }
}

use super::{MapBuilder, Map, TileType, Position, spawner, 
    SHOW_MAPGEN_VISUALIZER, remove_unreachable_areas_returning_most_distant, 
    generate_voronoi_spawn_regions};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm { Pythagoras, Manhattan, Chebyshev }

pub struct VoronoiBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    spawn_list: Vec<(usize, String)>,
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm
}

impl MapBuilder for VoronoiBuilder {
    fn build_map(&mut self)  {
        self.build();
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
    
}

impl VoronoiBuilder {
    pub fn new(new_depth: i32, n_seeds: usize, distance_algorithm: DistanceAlgorithm) -> VoronoiBuilder {
        VoronoiBuilder{
            map: Map::new(new_depth),
            starting_position: Position{ x: 0, y : 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            spawn_list: Vec::new(),
            n_seeds,
            distance_algorithm,
        }
    }

    pub fn pythagoras(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder::new(new_depth, 64, DistanceAlgorithm::Pythagoras)
    }

    pub fn manhattan(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder::new(new_depth, 64, DistanceAlgorithm::Manhattan)
    }

    pub fn chebyshev(new_depth: i32) -> VoronoiBuilder {
        VoronoiBuilder::new(new_depth, 128, DistanceAlgorithm::Chebyshev)
    }

    pub fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // manually make a voronoi diagram
        // store n_seeds random points on the map
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();
        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, self.map.width-1);
            let vy = rng.roll_dice(1, self.map.height-1);
            let vidx = self.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        // stores the distance from each seed (reused each iteration)
        let mut voronoi_distance = vec![(0, 0.0f32); self.n_seeds];
        // stores what seed each map tile belongs to
        let mut voronoi_membership: Vec<i32> = vec![0; self.map.width as usize * self.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % self.map.width;
            let y = i as i32 / self.map.width;
            // store distance from current tile to each seed tile
            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance;
                match self.distance_algorithm {
                    DistanceAlgorithm::Pythagoras => {
                        distance = rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(x, y),
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Manhattan => {
                        distance = rltk::DistanceAlg::Manhattan.distance2d(
                            rltk::Point::new(x, y),
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Chebyshev => {
                        distance = rltk::DistanceAlg::Chebyshev.distance2d(
                            rltk::Point::new(x, y),
                            pos.1
                        );
                    }
                }
                voronoi_distance[seed] = (seed, distance);

            }
            // set the tile's seed membership (vid) to the closest seed
            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let mut neighbours = 0;
                let my_idx = self.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                // check if neighbouring tiles belong to different seeds
                if voronoi_membership[self.map.xy_idx(x-1, y)] != my_seed { neighbours += 1; }
                if voronoi_membership[self.map.xy_idx(x+1, y)] != my_seed { neighbours += 1; }
                if voronoi_membership[self.map.xy_idx(x, y-1)] != my_seed { neighbours += 1; }
                if voronoi_membership[self.map.xy_idx(x, y+1)] != my_seed { neighbours += 1; }

                if neighbours < 2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                } // otherwise leave the tile as a wall
            }
            self.take_snapshot();
        }

        // find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }

        // cull unreachable areas and find an exit
        // TODO: culling doesn't work but placing exit does ?
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // build a noise map for spawning
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        for area in self. noise_areas.iter() {
            spawner::spawn_region(&self.map, &mut rng, area.1, self.depth, &mut self.spawn_list);
        }
    }
}

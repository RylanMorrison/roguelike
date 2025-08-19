use super::{InitialMapBuilder, BuilderMap, TileType};
use crate::rng;

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm { Pythagoras, Manhattan, Chebyshev }

pub struct VoronoiCellBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm
}

impl InitialMapBuilder for VoronoiCellBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap)  {
        self.build(build_data);
    }
}

impl VoronoiCellBuilder {
    pub fn new(n_seeds: usize, distance_algorithm: DistanceAlgorithm) -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds,
            distance_algorithm
        })
    }

    pub fn pythagoras() -> Box<VoronoiCellBuilder> {
        VoronoiCellBuilder::new(64, DistanceAlgorithm::Pythagoras)
    }

    pub fn manhattan() -> Box<VoronoiCellBuilder> {
        VoronoiCellBuilder::new( 64, DistanceAlgorithm::Manhattan)
    }

    pub fn chebyshev() -> Box<VoronoiCellBuilder> {
        VoronoiCellBuilder::new( 128, DistanceAlgorithm::Chebyshev)
    }

    pub fn build(&mut self, build_data: &mut BuilderMap) {
        // manually make a voronoi diagram
        // store n_seeds random points on the map
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();
        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng::roll_dice(1, build_data.map.width-1);
            let vy = rng::roll_dice(1, build_data.map.height-1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        // stores the distance from each seed (reused each iteration)
        let mut voronoi_distance = vec![(0, 0.0f32); self.n_seeds];
        // stores what seed each map tile belongs to
        let mut voronoi_membership: Vec<i32> = vec![0; build_data.map.width as usize * build_data.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % build_data.map.width;
            let y = i as i32 / build_data.map.width;
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

        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let mut neighbours = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                // check if neighbouring tiles belong to different seeds
                if voronoi_membership[build_data.map.xy_idx(x-1, y)] != my_seed { neighbours += 1; }
                if voronoi_membership[build_data.map.xy_idx(x+1, y)] != my_seed { neighbours += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y-1)] != my_seed { neighbours += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y+1)] != my_seed { neighbours += 1; }

                if neighbours < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                } // otherwise leave the tile as a wall
            }
        }
    }
}

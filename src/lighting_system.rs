use specs::prelude::*;
use super::{Viewshed, Position, Map, LightSource};
use rltk::{RGB, Point};

pub struct LightingSystem {}

impl<'a> System<'a> for LightingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, LightSource>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, viewshed, positions, lighting) = data;

        if map.outdoors {
            // no dark outdoors maps yet
            return;
        }

        let black = RGB::named(rltk::BLACK);
        for l in map.light.iter_mut() {
            *l = black;
        }

        for (viewshed, pos, light) in (&viewshed, &positions, &lighting).join() {
            let light_point = Point::new(pos.x, pos.y);
            let range_f = light.range as f32;
            for t in viewshed.visible_tiles.iter() {
                if t.x > 0 && t.x < map.width
                && t.y > 0 && t.y < map.height {
                    let idx = map.xy_idx(t.x, t.y);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(
                        light_point, *t
                    );
                    // lower intensity further away / scale to between 0 and 1
                    let intensity = (range_f - distance) / range_f;

                    map.light[idx] = map.light[idx] + (light.colour * intensity);
                }
            }
        }
    }
}

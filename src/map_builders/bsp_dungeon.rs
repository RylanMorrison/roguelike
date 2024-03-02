use super::{InitialMapBuilder, BuilderMap, Rect, TileType};
use rltk::RandomNumberGenerator;

pub struct BspDungeonBuilder {
    rects: Vec<Rect>
}

impl InitialMapBuilder for BspDungeonBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BspDungeonBuilder {
    pub fn new() -> Box<BspDungeonBuilder> {
        Box::new(BspDungeonBuilder { 
            rects: Vec::new()
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect> = Vec::new();
        self.rects.clear();
        // start with a single almost map sized rectangle
        self.rects.push(Rect::new(2, 2, build_data.map.width - 5, build_data.map.height - 5));
        let first_room = self.rects[0];
        self.add_subrects(first_room); // divide the first room

        // Get a random rectangle and divide it up to 240 times
        // Place a room and add it to the rooms list if it's possible to squeeze into the rectangle
        let mut n_rooms = 0;
        while n_rooms < 500 {
            let rect = self.get_random_rect(rng);
            let candidate = self.get_random_sub_rect(rect, rng);

            if self.is_possible(candidate, &build_data, &rooms) {
                rooms.push(candidate);
                self.add_subrects(rect);
            }
            n_rooms += 1;
        }
        build_data.rooms = Some(rooms);
    }

    /*
    ###############        ###############
    #             #        #  1   +   2  #
    #             #        #      +      #
    #      0      #   ->   #+++++++++++++#
    #             #        #   3  +   4  #
    #             #        #      +      #
    ###############        ###############
     */
    fn add_subrects(&mut self, rect: Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects.push(Rect::new(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(rect.x1, rect.y1 + half_height, half_width, half_height));
        self.rects.push(Rect::new(rect.x1 + half_width, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(rect.x1 + half_width, rect.y1 + half_height, half_width, half_height));
    }

    fn get_random_rect(&mut self, rng: &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    /*
    ###############        ########
    #             #        #   1  #
    #             #        #      #
    #      0      #   ->   ########
    #             #
    #             #
    ###############
     */
    fn get_random_sub_rect(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        // random width and height inside the rectangle, min 3 and max 10 tiles 
        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
        let h = i32::max(1, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;

        // randomly move the rectangle around a bit
        result.x1 += rng.roll_dice(1, 6)-1;
        result.y1 += rng.roll_dice(1, 6)-1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&mut self, rect: Rect, build_data: &BuilderMap, rooms: &Vec<Rect>) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for r in rooms.iter() {
            if r.intersect(&rect) { can_build = false; }
        }

        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                // check if within map bounds
                if x > build_data.map.width - 2 { can_build = false; }
                if y > build_data.map.height - 2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                
                if can_build {
                    // check for overlap with an existing room
                    let idx = build_data.map.xy_idx(x, y);
                    if build_data.map.tiles[idx] != TileType::Wall {
                        can_build = false;
                    }
                }
            }
        }
        can_build
    }
}


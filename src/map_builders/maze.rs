use super::{InitialMapBuilder, BuilderMap, TileType, Map};
use crate::rng;

pub struct MazeBuilder {}

impl InitialMapBuilder for MazeBuilder {
    fn build_map(&mut self, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl MazeBuilder {
    pub fn new() -> Box<MazeBuilder> {
        Box::new(MazeBuilder{})
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let mut maze = Grid::new((build_data.map.width / 2)-2, (build_data.map.height / 2)-2);
        maze.generate_maze(build_data);
    }
}

/* Maze code taken under MIT from https://github.com/cyucelen/mazeGenerator/ */

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell{
            row,
            column,
            walls: [true, true, true, true],
            visited: false
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

// 'a ensures that rng is not cleaned up before Grid
struct Grid {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize
}

impl Grid {
    fn new(width: i32, height: i32) -> Grid {
        let mut grid = Grid{
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0
        };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column));
            }
        }

        grid
    }
    
    fn calculate_index(&self, row: i32, column: i32) -> i32 {
        if row < 0 || column < 0 || column > self.width-1 || row > self.height-1 {
            -1
        } else {
            column + (row * self.width)
        }
    }

    fn get_available_neighbours(&self) -> Vec<usize> {
        let mut neighbours: Vec<usize> = Vec::new();

        let current_row = self.cells[self.current].row;
        let current_column = self.cells[self.current].column;

        let neighbour_indices: [i32; 4] = [
            self.calculate_index(current_row - 1, current_column),
            self.calculate_index(current_row, current_column + 1),
            self.calculate_index(current_row + 1, current_column),
            self.calculate_index(current_row, current_column - 1)
        ];

        for i in neighbour_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbours.push(*i as usize);
            }
        }

        neighbours
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbours = self.get_available_neighbours();
        if !neighbours.is_empty() {
            if neighbours.len() == 1 {
                return Some(neighbours[0]);
            } else {
                return Some(neighbours[(rng::roll_dice(1, neighbours.len() as i32)-1) as usize]);
            }
        }
        None
    }

    /*
        The first few iterations will get a non-visited neighbor, carving a clear path through the maze
        Each step along the way, the cell we've visited is added to backtrace
        This is effectively a drunken walk through the maze, but ensuring that we cannot return to a cell
        When we hit a point at which we have no neighbors (we've hit the end of the maze), the algorithm will change current to the first entry in our backtrace list
        It will then randomly walk from there, filling in more cells
        If that point can't go anywhere, it works back up the backtrace list
        This repeats until every cell has been visited, meaning that backtrace and neighbors are both empty
     */

    fn generate_maze(&mut self, build_data: &mut BuilderMap) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.cells[next].visited = true;
                    self.backtrace.push(self.current);
                    //   __lower_part__      __higher_part_
                    //   /            \      /            \
                    // --------cell1------ | cell2-----------
                    let (lower_part, higher_part) = 
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if !self.backtrace.is_empty() {
                        self.current = self.backtrace[0];
                        self.backtrace.remove(0);
                    } else {
                        break;
                    }
                }
            }

            if i % 50 == 0 {
                // only snapshot every 10th iteration
                self.copy_to_map(&mut build_data.map);
                build_data.take_snapshot();
            }
            i += 1;
        }
    }

    /*
        Normally each cell in the maze structure can have walls in any of the four directions
        For our structure walls aren't part of a tile they are a tile so need to fix this mismatch
        Double the size of the Grid and carve floors where walls aren't present
     */
    fn copy_to_map(&self, map: &mut Map) {
        for i in map.tiles.iter_mut() { *i = TileType::Wall; }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] { map.tiles[idx - map.width as usize] = TileType::Floor; }
            if !cell.walls[RIGHT] { map.tiles[idx + 1] = TileType::Floor; }
            if !cell.walls[BOTTOM] { map.tiles[idx + map.width as usize] = TileType::Floor; }
            if !cell.walls[LEFT] { map.tiles[idx - 1] = TileType::Floor; }
        }
    }
}

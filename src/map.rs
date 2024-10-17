use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Tile {
    pub passable: bool,
    pub neighbors: Vec<(usize, usize)>, // Stores coordinates of accessible neighbors
}

#[derive(Debug)]
pub struct Map {
    pub height: usize,
    pub width: usize,
    pub grid: Vec<Vec<Tile>>,
}

impl Map {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let _type = lines.next().unwrap()?;
        let height = lines.next().unwrap()?.split_whitespace().last().unwrap().parse::<usize>().unwrap();
        let width = lines.next().unwrap()?.split_whitespace().last().unwrap().parse::<usize>().unwrap();
        let _map = lines.next().unwrap()?;

        let mut grid = Vec::with_capacity(height);
        for line in lines.take(height) {
            let row: Vec<char> = line?.chars().collect();
            let tiles_row: Vec<Tile> = row.into_iter().map(|ch| Tile {
                passable: ch == '.',
                neighbors: Vec::new(), // Initially empty, to be filled later
            }).collect();
            grid.push(tiles_row);
        }

        let mut map = Map { height, width, grid };
        map.initialize_neighbors();

        Ok(map)
    }

    fn initialize_neighbors(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.grid[y][x].passable {
                    self.grid[y][x].neighbors = self.get_neighbors(x, y);
                }
            }
        }
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)]; // Up, down, left, right
        let mut neighbors = Vec::new();

        for &(dx, dy) in &directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            if new_x >= 0 && new_y >= 0 && new_x < self.width as i32 && new_y < self.height as i32 {
                if self.grid[new_y as usize][new_x as usize].passable {
                    neighbors.push((new_x as usize, new_y as usize));
                }
            }
        }

        neighbors
    }
}

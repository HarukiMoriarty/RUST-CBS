use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Tile {
    passable: bool,
    pub neighbors: Vec<(usize, usize)>, // Stores coordinates of accessible neighbors
}

impl Tile {
    pub fn is_passable(&self) -> bool {
        self.passable
    }
}

#[derive(Debug, Clone)]
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
        let height = lines
            .next()
            .unwrap()?
            .split_whitespace()
            .last()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let width = lines
            .next()
            .unwrap()?
            .split_whitespace()
            .last()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let _map = lines.next().unwrap()?;

        let mut grid = Vec::with_capacity(height);
        for line in lines.take(height) {
            let row: Vec<char> = line?.chars().collect();
            let tiles_row: Vec<Tile> = row
                .into_iter()
                .map(|ch| Tile {
                    passable: ch == '.',
                    neighbors: Vec::new(),
                })
                .collect();
            grid.push(tiles_row);
        }

        let mut map = Map {
            height,
            width,
            grid,
        };
        map.initialize_neighbors();

        Ok(map)
    }

    fn initialize_neighbors(&mut self) {
        for x in 0..self.height {
            for y in 0..self.width {
                if self.grid[x][y].passable {
                    self.grid[x][y].neighbors = self.get_neighbors(x, y);
                }
            }
        }
    }

    pub fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1), (0, 0)]; // Up, down, left, right, stay
        let mut neighbors = Vec::new();

        for &(dx, dy) in &directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            if new_x >= 0
                && new_y >= 0
                && new_x < self.height as i32
                && new_y < self.width as i32
                && self.grid[new_x as usize][new_y as usize].passable
            {
                neighbors.push((new_x as usize, new_y as usize));
            }
        }

        neighbors
    }

    pub fn is_passable(&self, x: usize, y: usize) -> bool {
        self.grid[x][y].is_passable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_map() {
        let map = Map::from_file("map_file/maze-32-32-2-scen-even/maze-32-32-2.map").unwrap();

        assert_eq!(map.height, 32);
        assert_eq!(map.width, 32);

        assert!(!map.is_passable(0, 0));
        assert!(!map.is_passable(1, 0));
        assert!(!map.is_passable(0, 1));
        assert!(map.is_passable(1, 1));

        let neighbors = map.get_neighbors(1, 1);
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&(2, 1)));
        assert!(neighbors.contains(&(1, 2)));
    }
}

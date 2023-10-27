use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

enum CellState {
    Alive,
    Dead,
}

pub struct LifeIteration {
    pub cells: HashSet<Coords>,
}

impl Coords {
    fn new(x: isize, y: isize) -> Self {
        Coords { x, y }
    }

    fn get_neighbor_coords(&self) -> [Coords; 8] {
        let (x, y) = (self.x, self.y);
        [
            Coords::new(x - 1, y + 1),
            Coords::new(x - 1, y),
            Coords::new(x - 1, y - 1),
            Coords::new(x, y - 1),
            Coords::new(x, y + 1),
            Coords::new(x + 1, y + 1),
            Coords::new(x + 1, y),
            Coords::new(x + 1, y - 1),
        ]
    }

    fn get_next_state(alive_neighbors_count: usize, cur_state: CellState) -> CellState {
        match (alive_neighbors_count, cur_state) {
            (n, state) => match state {
                CellState::Alive => {
                    if n == 2 || n == 3 {
                        CellState::Alive
                    } else {
                        CellState::Dead
                    }
                }
                CellState::Dead => {
                    if n == 3 {
                        CellState::Alive
                    } else {
                        CellState::Dead
                    }
                }
            },
        }
    }
}

impl LifeIteration {
    pub fn get_next_life_iteration(self) -> LifeIteration {
        let mut new_cells: HashSet<Coords> = HashSet::new();

        for cell_key in self.cells.iter() {
            let neighbor_coords = cell_key.get_neighbor_coords();
            let alive_neighbors_count = neighbor_coords
                .iter()
                .filter(|&coord| self.cells.get(coord).is_some())
                .count();
            let next_state = Coords::get_next_state(alive_neighbors_count, CellState::Alive);

            if let CellState::Alive = next_state {
                let coords = Coords {
                    x: cell_key.x,
                    y: cell_key.y,
                };
                new_cells.insert(coords);
            }

            for neighbor in neighbor_coords
                .iter()
                .filter(|&coord| self.cells.get(coord).is_none())
            {
                let neighbor_coords = neighbor.get_neighbor_coords();
                let alive_neighbors_count = neighbor_coords
                    .iter()
                    .filter(|&coord| self.cells.get(coord).is_some())
                    .count();
                let next_state = Coords::get_next_state(alive_neighbors_count, CellState::Dead);

                if let CellState::Alive = next_state {
                    let coords = Coords {
                        x: neighbor.x,
                        y: neighbor.y,
                    };
                    new_cells.insert(coords);
                }
            }
        }

        LifeIteration { cells: new_cells }
    }
}

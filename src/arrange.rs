#[derive(Debug, Default, Clone)]
pub struct CellSize {
    start_row: usize,
    start_col: usize,
    width: usize,
    height: usize,
    card_type: String,
}


pub fn arrange_grid(grid_size: (usize, usize), cell_list: &[String]) -> Vec<CellSize> {

    let (grow, gcol) = grid_size;

    let mut grid = vec![vec![' '; gcol]; grow];
    let mut start_row = 0;

    let mut cell_size_list = vec![];

    for cell in cell_list.iter() {
        let (cell_type, cell_shape) = cell.split_once('-').unwrap();
        let cell_size = add_cell(&mut grid, cell_shape, cell_type, 'x', start_row);
        if cell_size.start_col == 999 { continue; }
        start_row = cell_size.start_row;
        cell_size_list.push(cell_size);
    }

    // let grid_str = grid
       // .into_iter()
        //.filter(|row| !row.iter().all(|c| *c == ' '))
        // .map(|row| row.into_iter().collect::<String>())
        // .collect::<Vec<_>>()
        // .join("\n");

    // println!("{}", grid_str);

    cell_size_list

    // grid
}



impl CellSize {
    pub fn get_start_row(&self) -> usize {
        self.start_row
    }

    pub fn get_start_col(&self) -> usize {
        self.start_col
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_card_type(&self) -> &String {
        &self.card_type
    }
}


fn can_place_cell(grid: &[Vec<char>], row: usize, col: usize, width: usize, height: usize) -> bool {
    for r in row..row + height {
        for c in col..col + width {
            if r >= grid.len() || c >= grid[0].len() || grid[r][c] != ' ' {
                return false;
            }
        }
    }
    true
}

fn place_cell(grid: &mut [Vec<char>], row: usize, col: usize, width: usize, height: usize, char: char) {
    for r in row..row + height {
        for c in col..col + width {
            grid[r][c] = char;
        }
    }
}

fn try_place_cell(grid: &mut [Vec<char>], width: usize, height: usize, char: char, start_row: usize) -> (usize, usize) {
    for row in start_row..grid.len() {
        for col in 0..=grid[0].len() - width {
            if can_place_cell(grid, row, col, width, height) {
                place_cell(grid, row, col, width, height, char);
                return (row, col);
            }
        }
    }
    (start_row, 999)
}

fn add_cell(grid: &mut [Vec<char>], cell_shape: &str, cell_type: &str, cell_char: char, start_row: usize) -> CellSize {
    let (num1, num2) = cell_shape.split_once('x').unwrap();
    let num1: usize = num1.parse().unwrap();
    let num2: usize = num2.parse().unwrap();
    let (start_row, start_col) = try_place_cell(grid, num2, num1, cell_char, start_row);
    CellSize {
        start_row,
        start_col,
        width: num2,
        height: num1,
        card_type: cell_type.to_string(),
    }
}

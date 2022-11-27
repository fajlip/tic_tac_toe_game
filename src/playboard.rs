use crate::matrix_display;
use matrix_display::*;

use crate::cli_args_processing::StartOrder;
use crate::settings::playboard_options::{
    PLAYBOARD_COLOR_TEXT, PLAYBOARD_GRID_COLOR1, PLAYBOARD_GRID_COLOR2, PLAYBOARD_GRID_HEIGHT,
    PLAYBOARD_GRID_WIDTH, PLAYBOARD_ROW_COL_SIZE, PLAYBOARD_SIZE,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PlayBoardGridOptions {
    X,
    O,
    Free,
}

pub enum GameState {
    InvalidPlace,
    Placed,
    Draw,
    GameOver,
}

pub struct Playboard {
    grid: [PlayBoardGridOptions; PLAYBOARD_SIZE],
}

impl Playboard {
    pub fn new() -> Self {
        if PLAYBOARD_SIZE % 2 != 1 {
            panic!("Invalid playboard size settings, must be odd number.");
        }

        if PLAYBOARD_SIZE * PLAYBOARD_SIZE == PLAYBOARD_ROW_COL_SIZE {
            panic!("Invalid playboard size settings, power of playboard row col size.");
        }

        // Initialize grid with free option.
        let grid: [PlayBoardGridOptions; PLAYBOARD_SIZE] =
            [PlayBoardGridOptions::Free; PLAYBOARD_SIZE];

        Self { grid }
    }

    fn i2d_into_1d(row: usize, col: usize) -> usize {
        row * PLAYBOARD_ROW_COL_SIZE + col
    }

    fn check_validity_of_indexes(&self, row: usize, col: usize) -> bool {
        row <= PLAYBOARD_ROW_COL_SIZE
            && row > 0
            && col <= PLAYBOARD_ROW_COL_SIZE
            && col > 0
            && self.grid[Self::i2d_into_1d(row - 1, col - 1)] == PlayBoardGridOptions::Free
    }

    fn check_if_same_symbols(items: Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]>) -> bool
    {
        for item in &items {
            let tmp = item
                .iter()
                .zip(item.iter().skip(1))
                .map(|(&x, &y)| x == y && x != PlayBoardGridOptions::Free)
                .collect::<Vec<bool>>();

            // todo: misto collect
            if tmp.iter().all(|&i| i) {
                return true;
            }
        }

        false
    }

    fn check_for_game_win(&self) -> bool {
        let row_items = self.get_row_items();
        let col_items = self.get_col_items();
        let diagonal_items = self.get_diagonal_items();

        // Check for win on rows, cols and diagonal.
        if Self::check_if_same_symbols(row_items)
            || Self::check_if_same_symbols(col_items)
            || Self::check_if_same_symbols(diagonal_items)
        {
            return true;
        }

        false
    }

    // k zamysleni: drzet pocet plnych poli
    fn check_for_full_playboard(&self) -> bool {
        for index_grid in 0..self.grid.len() {
            if self.grid[index_grid] == PlayBoardGridOptions::Free {
                return false;
            }
        }

        true
    }

    pub fn place_on_grid(&mut self, row: usize, col: usize, start_order: StartOrder) -> GameState {
        if !self.check_validity_of_indexes(row, col) {
            return GameState::InvalidPlace;
        }

        // Players index from 1.
        let row = row - 1;
        let col = col - 1;

        let player_playboard_grid_option = match start_order {
            StartOrder::First => PlayBoardGridOptions::X,
            StartOrder::Second => PlayBoardGridOptions::O,
        };

        self.grid[Self::i2d_into_1d(row, col)] = player_playboard_grid_option;

        if self.check_for_game_win() {
            GameState::GameOver
        } else if self.check_for_full_playboard() {
            GameState::Draw
        } else {
            GameState::Placed
        }
    }

    fn tranfer_playboard_grid_options_to_printable(&self) -> [char; PLAYBOARD_SIZE] {
        let mut grid_printable: [char; PLAYBOARD_SIZE] = [' '; PLAYBOARD_SIZE];

        for index_grid in 0..self.grid.len() {
            let printable_symbol = match self.grid[index_grid] {
                PlayBoardGridOptions::X => 'X',
                PlayBoardGridOptions::O => 'O',
                PlayBoardGridOptions::Free => ' ',
            };

            grid_printable[index_grid] = printable_symbol;
        }

        grid_printable
    }

    pub fn display_board(&self) {
        let format = Format::new(PLAYBOARD_GRID_WIDTH, PLAYBOARD_GRID_HEIGHT);

        let grid_printable = self.tranfer_playboard_grid_options_to_printable();

        let board = grid_printable
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let mut color_grid = PLAYBOARD_GRID_COLOR1;
                if i % 2 == 1 {
                    color_grid = PLAYBOARD_GRID_COLOR2;
                }
                cell::Cell::new(*x, PLAYBOARD_COLOR_TEXT, color_grid)
            })
            .collect::<Vec<_>>();

        let mut data = matrix::Matrix::new(PLAYBOARD_ROW_COL_SIZE, board);
        let display = MatrixDisplay::new(&format, &mut data);
        display.print(&mut std::io::stdout(), &style::BordersStyle::None);
    }

    pub fn clear_board(&mut self) {
        self.grid = [PlayBoardGridOptions::Free; PLAYBOARD_SIZE];
    }

    // TODO: Predelat

    fn get_row_items(&self) -> Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]> {
        let mut grid_rows: Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]> = Vec::new();

        for index_row in 0..PLAYBOARD_ROW_COL_SIZE {
            let mut row_values = [PlayBoardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];

            for index_col in 0..PLAYBOARD_ROW_COL_SIZE {
                row_values[index_col] = self.grid[Self::i2d_into_1d(index_row, index_col)];
            }

            grid_rows.push(row_values);
        }

        grid_rows
    }

    fn get_col_items(&self) -> Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]> {
        let mut grid_cols: Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]> = Vec::new();

        for index_col in 0..PLAYBOARD_ROW_COL_SIZE {
            let mut col_values = [PlayBoardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];

            for index_row in 0..PLAYBOARD_ROW_COL_SIZE {
                col_values[index_row] = self.grid[Self::i2d_into_1d(index_row, index_col)];
            }

            grid_cols.push(col_values);
        }

        grid_cols
    }

    fn get_diagonal_items(&self) -> Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]> {
        let mut grid_diagonal: Vec<[PlayBoardGridOptions; PLAYBOARD_ROW_COL_SIZE]> = Vec::new();

        let mut main_diagonal_values = [PlayBoardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];
        let mut main_diagnoal_array_index = 0;
        for index_row in 0..PLAYBOARD_ROW_COL_SIZE {
            for index_col in 0..PLAYBOARD_ROW_COL_SIZE {
                if index_row == index_col {
                    main_diagonal_values[main_diagnoal_array_index] =
                        self.grid[Self::i2d_into_1d(index_row, index_col)];
                    main_diagnoal_array_index += 1;
                }
            }
        }

        grid_diagonal.push(main_diagonal_values);

        let mut anti_diagonal_values = [PlayBoardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];
        let mut anti_diagnoal_array_index = 0;
        for index_row in 0..PLAYBOARD_ROW_COL_SIZE {
            for index_col in 0..PLAYBOARD_ROW_COL_SIZE {
                if index_row + index_col == PLAYBOARD_ROW_COL_SIZE - 1 {
                    anti_diagonal_values[anti_diagnoal_array_index] =
                        self.grid[Self::i2d_into_1d(index_row, index_col)];
                    anti_diagnoal_array_index += 1;
                }
            }
        }

        grid_diagonal.push(anti_diagonal_values);

        grid_diagonal
    }
}

use crate::matrix_display;
use matrix_display::*;

use std::iter::{Skip, StepBy, Take};
use std::{fmt, sync::MutexGuard};

use crate::cli_args_processing::StartOrder;
use crate::settings::playboard_options::{
    PLAYBOARD_COLOR_TEXT, PLAYBOARD_GRID_COLOR1, PLAYBOARD_GRID_COLOR2, PLAYBOARD_GRID_HEIGHT,
    PLAYBOARD_GRID_WIDTH, PLAYBOARD_ROW_COL_SIZE, PLAYBOARD_SIZE,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PlayboardGridOptions {
    X,
    O,
    Free,
}

impl fmt::Display for PlayboardGridOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PlayboardGridOptions::X => write!(f, "X"),
            PlayboardGridOptions::O => write!(f, "O"),
            PlayboardGridOptions::Free => write!(f, " "),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    InvalidPlace,
    Placed,
    Draw,
    GameOver,
}

#[derive(Debug, PartialEq)]
pub struct Playboard {
    grid: [PlayboardGridOptions; PLAYBOARD_SIZE],
}

pub struct InvalidPlayboardSize<'a> {
    pub error: &'a str,
}

impl<'a> fmt::Display for InvalidPlayboardSize<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl Playboard {
    pub fn new() -> Result<Self, InvalidPlayboardSize<'static>> {
        if PLAYBOARD_SIZE % 2 != 1 {
            return Err(InvalidPlayboardSize {
                error: "Playboard size must be odd number.",
            });
        } else if PLAYBOARD_ROW_COL_SIZE * PLAYBOARD_ROW_COL_SIZE != PLAYBOARD_SIZE {
            return Err(InvalidPlayboardSize {
                error: "Playboard size does not correspond to playboard row and col size.",
            });
        }

        // Initialize grid with free option.
        let grid: [PlayboardGridOptions; PLAYBOARD_SIZE] =
            [PlayboardGridOptions::Free; PLAYBOARD_SIZE];

        Ok(Self { grid })
    }

    fn i2d_into_1d(row: usize, col: usize) -> usize {
        row * PLAYBOARD_ROW_COL_SIZE + col
    }

    fn check_validity_of_indexes(&self, row: usize, col: usize) -> bool {
        row < PLAYBOARD_ROW_COL_SIZE
            && col < PLAYBOARD_ROW_COL_SIZE
            && self.grid[Self::i2d_into_1d(row, col)] == PlayboardGridOptions::Free
    }

    fn check_if_same_symbols(items: Vec<[PlayboardGridOptions; PLAYBOARD_ROW_COL_SIZE]>) -> bool {
        // todo: dalsi iterator a any
        for item in &items {
            if item
                .iter()
                .zip(item.iter().skip(1))
                .map(|(&x, &y)| x == y && x != PlayboardGridOptions::Free)
                .all(|i| i)
            {
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
        Self::check_if_same_symbols(row_items)
            || Self::check_if_same_symbols(col_items)
            || Self::check_if_same_symbols(diagonal_items)
    }

    // k zamysleni: drzet pocet plnych poli
    fn check_for_full_playboard(&self) -> bool {
        self.grid.iter().all(|&i| i != PlayboardGridOptions::Free)
    }

    // todo: vracet result a zacinat od nuly
    pub fn place_on_grid(&mut self, row: usize, col: usize, start_order: StartOrder) -> GameState {
        // Players index from 1.
        assert!(row > 0);
        assert!(col > 0);
        let row: usize = row - 1;
        let col: usize = col - 1;

        if !self.check_validity_of_indexes(row, col) {
            return GameState::InvalidPlace;
        }

        let player_playboard_grid_option = match start_order {
            StartOrder::First => PlayboardGridOptions::X,
            StartOrder::Second => PlayboardGridOptions::O,
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

    pub fn clear_board(&mut self) {
        self.grid = [PlayboardGridOptions::Free; PLAYBOARD_SIZE];
    }

    // TODO: Predelat

    fn get_row_items(&self) -> Vec<[PlayboardGridOptions; PLAYBOARD_ROW_COL_SIZE]> {
        (*self
            .grid
            .array_chunks::<PLAYBOARD_ROW_COL_SIZE>()
            .cloned()
            .collect::<Vec<_>>())
        .to_vec()
    }

    fn get_row_iter(&self, row: usize) -> Take<Skip<std::slice::Iter<'_, PlayboardGridOptions>>> {
        assert!(row < PLAYBOARD_ROW_COL_SIZE);
        self.grid
            .iter()
            .skip(row * PLAYBOARD_ROW_COL_SIZE)
            .take(PLAYBOARD_ROW_COL_SIZE)
    }

    fn get_col_iter(&self, col: usize) -> StepBy<Skip<std::slice::Iter<'_, PlayboardGridOptions>>> {
        assert!(col < PLAYBOARD_ROW_COL_SIZE);
        self.grid.iter().skip(col).step_by(PLAYBOARD_ROW_COL_SIZE)
    }

    fn get_main_diag_iter(&self) -> impl Iterator<Item = &PlayboardGridOptions> {
        let i1d_into_2d = |i: usize| -> (usize, usize) {
            (
                (i / PLAYBOARD_ROW_COL_SIZE) as usize,
                i % PLAYBOARD_ROW_COL_SIZE,
            )
        };

        self.grid
            .iter()
            .enumerate()
            .filter(move |&(i, _)| {
                let (row, col) = i1d_into_2d(i);
                row == col
            })
            .map(|(_, e)| e)
    }

    fn get_anti_diag_iter(&self) -> impl Iterator<Item = &PlayboardGridOptions> {
        let i1d_into_2d = |i: usize| -> (usize, usize) {
            (
                (i / PLAYBOARD_ROW_COL_SIZE) as usize,
                i % PLAYBOARD_ROW_COL_SIZE,
            )
        };

        self.grid
            .iter()
            .enumerate()
            .filter(move |&(i, _)| {
                let (row, col) = i1d_into_2d(i);
                row + col == PLAYBOARD_ROW_COL_SIZE - 1
            })
            .map(|(_, e)| e)
    }

    fn get_col_items(&self) -> Vec<[PlayboardGridOptions; PLAYBOARD_ROW_COL_SIZE]> {
        let mut grid_cols: Vec<[PlayboardGridOptions; PLAYBOARD_ROW_COL_SIZE]> = Vec::new();

        for index_col in 0..PLAYBOARD_ROW_COL_SIZE {
            let mut col_values = [PlayboardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];

            for index_row in 0..PLAYBOARD_ROW_COL_SIZE {
                col_values[index_row] = self.grid[Self::i2d_into_1d(index_row, index_col)];
            }

            grid_cols.push(col_values);
        }

        grid_cols
    }

    fn get_diagonal_items(&self) -> Vec<[PlayboardGridOptions; PLAYBOARD_ROW_COL_SIZE]> {
        let mut grid_diagonal: Vec<[PlayboardGridOptions; PLAYBOARD_ROW_COL_SIZE]> = Vec::new();

        let mut main_diagonal_values = [PlayboardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];
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

        let mut anti_diagonal_values = [PlayboardGridOptions::Free; PLAYBOARD_ROW_COL_SIZE];
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

pub fn display_board(playboard: MutexGuard<Playboard>) {
    let format = Format::new(PLAYBOARD_GRID_WIDTH, PLAYBOARD_GRID_HEIGHT);

    let board = playboard
        .grid
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

#[cfg(test)]
mod tests {
    use crate::playboard;
    use playboard::*;

    #[test]
    fn test_new() {
        let expected_result = [PlayboardGridOptions::Free; PLAYBOARD_SIZE];

        let result = match Playboard::new() {
            Ok(playboard) => playboard,
            Err(_) => {
                assert!(false);
                return;
            }
        };

        assert_eq!(result.grid, expected_result);
        assert_eq!(result.grid.len(), PLAYBOARD_SIZE);
    }

    #[rustfmt::skip]
    fn prepare_playboard() -> Playboard {
        Playboard {
            grid:
            [
                PlayboardGridOptions::Free, PlayboardGridOptions::X,    PlayboardGridOptions::X,
                PlayboardGridOptions::X,    PlayboardGridOptions::Free, PlayboardGridOptions::O,
                PlayboardGridOptions::O,    PlayboardGridOptions::Free, PlayboardGridOptions::X,
            ],
        }
    }

    #[test]
    fn test_i2d_into_1d() {
        assert_eq!(Playboard::i2d_into_1d(0, 0), 0);
        assert_eq!(Playboard::i2d_into_1d(0, 2), 2);
        assert_eq!(Playboard::i2d_into_1d(1, 0), 3);
        assert_eq!(Playboard::i2d_into_1d(1, 1), 4);
        assert_eq!(Playboard::i2d_into_1d(1, 2), 5);
        assert_eq!(Playboard::i2d_into_1d(2, 0), 6);
        assert_eq!(Playboard::i2d_into_1d(2, 1), 7);
        assert_eq!(Playboard::i2d_into_1d(2, 2), 8);
    }

    #[test]
    fn test_check_validity_of_indexes() {
        let playboard = prepare_playboard();

        // Free fields:
        assert!(playboard.check_validity_of_indexes(0, 0));
        assert!(playboard.check_validity_of_indexes(1, 1));
        assert!(playboard.check_validity_of_indexes(2, 1));
        // Invalid row:
        assert!(!playboard.check_validity_of_indexes(4, 0));
        // Invalid col:
        assert!(!playboard.check_validity_of_indexes(0, 4));
        // Occupied fields:
        assert!(!playboard.check_validity_of_indexes(0, 1));
        assert!(!playboard.check_validity_of_indexes(1, 2));
        assert!(!playboard.check_validity_of_indexes(2, 2));
    }

    #[test]
    #[rustfmt::skip]
    fn test_check_if_same_symbols() {
        // Free option is not considered as same symbol:
        assert!(!Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::Free, PlayboardGridOptions::Free, PlayboardGridOptions::Free]]));
        
        assert!(Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::X, PlayboardGridOptions::X, PlayboardGridOptions::X]]));
        assert!(Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::O, PlayboardGridOptions::O, PlayboardGridOptions::O]]));

        assert!(!Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::X, PlayboardGridOptions::O, PlayboardGridOptions::X]]));
        assert!(!Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::O, PlayboardGridOptions::O, PlayboardGridOptions::X]]));
        assert!(!Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::X, PlayboardGridOptions::X, PlayboardGridOptions::Free]]));
        assert!(!Playboard::check_if_same_symbols(vec![[PlayboardGridOptions::O, PlayboardGridOptions::O, PlayboardGridOptions::Free]]));
    }

    #[test]
    fn test_check_for_game_win() {
        // todo
    }

    #[test]
    fn test_check_for_full_playboard() {
        let mut playboard = prepare_playboard();

        assert!(!playboard.check_for_full_playboard());

        playboard.grid[0] = PlayboardGridOptions::X;
        assert!(!playboard.check_for_full_playboard());

        playboard.grid[4] = PlayboardGridOptions::O;
        assert!(!playboard.check_for_full_playboard());

        playboard.grid[7] = PlayboardGridOptions::O;
        assert!(playboard.check_for_full_playboard());
    }

    #[test]
    fn test_invalid_place_on_grid() {
        let mut playboard = prepare_playboard();

        // Invalid place options.
        // Row out of board:
        assert_eq!(
            playboard.place_on_grid(4, 3, StartOrder::First),
            GameState::InvalidPlace
        );
        // Col out of board:
        assert_eq!(
            playboard.place_on_grid(3, 4, StartOrder::First),
            GameState::InvalidPlace
        );
        // Field is not free:
        assert_eq!(
            playboard.place_on_grid(1, 2, StartOrder::First),
            GameState::InvalidPlace
        );

        // Playboard unchanged.
        assert_eq!(playboard, prepare_playboard());
    }

    #[test]
    fn test_place_on_grid_game_running() {
        let mut playboard = prepare_playboard();

        // Placed but game running options:
        assert_eq!(
            playboard.place_on_grid(1, 1, StartOrder::Second),
            GameState::Placed
        );
        assert_eq!(playboard.grid[0], PlayboardGridOptions::O);

        assert_eq!(
            playboard.place_on_grid(3, 2, StartOrder::First),
            GameState::Placed
        );
        assert_eq!(playboard.grid[7], PlayboardGridOptions::X);
    }

    #[test]
    fn test_place_on_grid_game_ended() {
        let mut playboard = prepare_playboard();
        playboard.grid[0] = PlayboardGridOptions::O;
        playboard.grid[7] = PlayboardGridOptions::X;

        // Game not running anymore options:
        assert_eq!(
            playboard.place_on_grid(2, 2, StartOrder::Second),
            GameState::Draw
        );
        assert_eq!(playboard.grid[4], PlayboardGridOptions::O);

        // Winning in row.
        playboard.grid[1] = PlayboardGridOptions::O;
        playboard.grid[2] = PlayboardGridOptions::Free;
        assert_eq!(
            playboard.place_on_grid(1, 3, StartOrder::Second),
            GameState::GameOver
        );

        // Winning in col.
        playboard.grid[1] = PlayboardGridOptions::X;
        playboard.grid[4] = PlayboardGridOptions::Free;
        assert_eq!(
            playboard.place_on_grid(2, 2, StartOrder::First),
            GameState::GameOver
        );
    }

    #[test]
    fn test_clear_board() {
        let mut playboard = prepare_playboard();

        let expected_result = [PlayboardGridOptions::Free; PLAYBOARD_SIZE];

        playboard.clear_board();

        assert_eq!(playboard.grid, expected_result);
    }

    #[test]
    #[rustfmt::skip]
    fn test_get_row_items() {
        let expected_result = vec![
            [PlayboardGridOptions::Free, PlayboardGridOptions::X,    PlayboardGridOptions::X],
            [PlayboardGridOptions::X,    PlayboardGridOptions::Free, PlayboardGridOptions::O],
            [PlayboardGridOptions::O,    PlayboardGridOptions::Free, PlayboardGridOptions::X],
        ];
        
        let playboard = prepare_playboard();

        let result = playboard.get_row_items();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[rustfmt::skip]
    fn test_get_col_items() {
        let expected_result = vec![
            [PlayboardGridOptions::Free, PlayboardGridOptions::X,    PlayboardGridOptions::O],
            [PlayboardGridOptions::X,    PlayboardGridOptions::Free, PlayboardGridOptions::Free],
            [PlayboardGridOptions::X,    PlayboardGridOptions::O,    PlayboardGridOptions::X],
        ];
        
        let playboard = prepare_playboard();

        let result = playboard.get_col_items();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[rustfmt::skip]
    fn test_get_diagonal_items() {
        let expected_result = vec![
            [PlayboardGridOptions::Free, PlayboardGridOptions::Free, PlayboardGridOptions::X],
            [PlayboardGridOptions::X,    PlayboardGridOptions::Free, PlayboardGridOptions::O],
        ];
        
        let playboard = prepare_playboard();

        let result = playboard.get_diagonal_items();

        assert_eq!(result, expected_result);
    }
}

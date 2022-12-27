use crate::matrix_display;
use matrix_display::*;

use std::iter::{Skip, StepBy, Take};
use std::{fmt, sync::MutexGuard};

use crate::cli_args_processing::StartOrder;
use crate::settings::playboard_options::{
    PLAYBOARD_COLOR_TEXT, PLAYBOARD_GRID_COLOR1, PLAYBOARD_GRID_COLOR2, PLAYBOARD_GRID_HEIGHT,
    PLAYBOARD_GRID_WIDTH,
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

fn i2d_into_1d(row: usize, col: usize, number_of_cols: usize) -> usize {
    row * number_of_cols + col
}

fn i1d_into_2d(index: usize, number_of_cols: usize) -> (usize, usize) {
    (
        (index / number_of_cols) as usize,
        index % number_of_cols,
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct Playboard {
    grid: Vec<PlayboardGridOptions>,
    row_col_size: usize,
    size: usize,
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
    pub fn new<const ROW_COL_SIZE: usize>() -> Result<Self, InvalidPlayboardSize<'static>> {
        let size: usize = ROW_COL_SIZE * ROW_COL_SIZE;

        if size % 2 != 1 {
            return Err(InvalidPlayboardSize {
                error: "Playboard size must be odd number.",
            });
        }

        // Initialize grid with free option.
        let grid: Vec<PlayboardGridOptions> = vec![PlayboardGridOptions::Free; size];

        Ok(Self { grid, row_col_size: ROW_COL_SIZE, size })
    }

    fn check_validity_of_indexes(&self, row: usize, col: usize) -> bool {
        row < self.row_col_size
            && col < self.row_col_size
            && self.grid[i2d_into_1d(row, col, self.row_col_size)] == PlayboardGridOptions::Free
    }

    // todo: vylepsit osetreni na none
    fn check_if_same_symbols<'a, T>(mut data_iter: T) -> bool
    where
    T: Iterator<Item = &'a PlayboardGridOptions>,
    {
        match data_iter.next() {
            Some(first) => data_iter.all(|&x| x != PlayboardGridOptions::Free && x == *first),
            None => false,
        }
    }

    fn check_for_game_win(&self, row: usize, col: usize) -> bool {
        // Check for win on rows, cols and diagonal.
        Self::check_if_same_symbols(self.get_row_iter(row))
            || Self::check_if_same_symbols(self.get_col_iter(col))
            || Self::check_if_same_symbols(self.get_main_diag_iter())
            || Self::check_if_same_symbols(self.get_anti_diag_iter())
    }

    // k zamysleni: drzet pocet plnych poli
    fn check_for_full_playboard(&self) -> bool {
        self.grid.iter().all(|&i| i != PlayboardGridOptions::Free)
    }

    // todo: vracet result
    pub fn place_on_grid(&mut self, row: usize, col: usize, start_order: StartOrder) -> GameState {
        if !self.check_validity_of_indexes(row, col) {
            return GameState::InvalidPlace;
        }

        let player_playboard_grid_option = match start_order {
            StartOrder::First => PlayboardGridOptions::X,
            StartOrder::Second => PlayboardGridOptions::O,
        };

        self.grid[i2d_into_1d(row, col, self.row_col_size)] = player_playboard_grid_option;

        if self.check_for_game_win(row, col) {
            GameState::GameOver
        } else if self.check_for_full_playboard() {
            GameState::Draw
        } else {
            GameState::Placed
        }
    }

    pub fn clear_board(&mut self) {
        self.grid = vec![PlayboardGridOptions::Free; self.size];
    }

    fn get_row_iter(&self, row: usize) -> Take<Skip<std::slice::Iter<'_, PlayboardGridOptions>>> {
        assert!(row < self.row_col_size);
        self.grid
            .iter()
            .skip(row * self.row_col_size)
            .take(self.row_col_size)
    }

    fn get_col_iter(&self, col: usize) -> StepBy<Skip<std::slice::Iter<'_, PlayboardGridOptions>>> {
        assert!(col < self.row_col_size);
        self.grid.iter().skip(col).step_by(self.row_col_size)
    }

    fn get_main_diag_iter(&self) -> impl Iterator<Item = &PlayboardGridOptions> {
        self.grid
            .iter()
            .enumerate()
            .filter(move |&(i, _)| {
                let (row, col) = i1d_into_2d(i, self.row_col_size);
                row == col
            })
            .map(|(_, e)| e)
    }

    fn get_anti_diag_iter(&self) -> impl Iterator<Item = &PlayboardGridOptions> {
        self.grid
            .iter()
            .enumerate()
            .filter(move |&(i, _)| {
                let (row, col) = i1d_into_2d(i, self.row_col_size);
                row + col == self.row_col_size - 1
            })
            .map(|(_, e)| e)
    }
}

pub fn display_board(playboard: MutexGuard<Playboard>, row_col_size: usize) {
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

    let mut data = matrix::Matrix::new(row_col_size, board);
    let display = MatrixDisplay::new(&format, &mut data);
    display.print(&mut std::io::stdout(), &style::BordersStyle::None);
}

#[cfg(test)]
mod tests {
    use crate::playboard;
    use playboard::*;

    const TESTED_PLAYBOARD_ROW_COL_SIZE: usize = 3;
    const TESTED_PLAYBOARD_SIZE: usize = 9;

    #[test]
    fn test_new() {
        let expected_result = [PlayboardGridOptions::Free; TESTED_PLAYBOARD_SIZE];

        let result = match Playboard::new::<TESTED_PLAYBOARD_ROW_COL_SIZE>() {
            Ok(playboard) => playboard,
            Err(_) => {
                assert!(false);
                return;
            }
        };

        assert_eq!(result.grid, expected_result);
        assert_eq!(result.grid.len(), TESTED_PLAYBOARD_SIZE);
    }

    #[rustfmt::skip]
    fn prepare_playboard() -> Playboard {
        Playboard {
            grid:
            vec![
                PlayboardGridOptions::Free, PlayboardGridOptions::X,    PlayboardGridOptions::X,
                PlayboardGridOptions::X,    PlayboardGridOptions::Free, PlayboardGridOptions::O,
                PlayboardGridOptions::O,    PlayboardGridOptions::Free, PlayboardGridOptions::X,
            ],
            row_col_size: TESTED_PLAYBOARD_ROW_COL_SIZE,
            size: TESTED_PLAYBOARD_SIZE
        }
    }

    #[test]
    fn test_i2d_into_1d() {
        assert_eq!(i2d_into_1d(0, 0, TESTED_PLAYBOARD_ROW_COL_SIZE), 0);
        assert_eq!(i2d_into_1d(0, 2, TESTED_PLAYBOARD_ROW_COL_SIZE), 2);
        assert_eq!(i2d_into_1d(1, 0, TESTED_PLAYBOARD_ROW_COL_SIZE), 3);
        assert_eq!(i2d_into_1d(1, 1, TESTED_PLAYBOARD_ROW_COL_SIZE), 4);
        assert_eq!(i2d_into_1d(1, 2, TESTED_PLAYBOARD_ROW_COL_SIZE), 5);
        assert_eq!(i2d_into_1d(2, 0, TESTED_PLAYBOARD_ROW_COL_SIZE), 6);
        assert_eq!(i2d_into_1d(2, 1, TESTED_PLAYBOARD_ROW_COL_SIZE), 7);
        assert_eq!(i2d_into_1d(2, 2, TESTED_PLAYBOARD_ROW_COL_SIZE), 8);
    }

    #[test]
    fn test_i1d_into_2d() {
        assert_eq!(i1d_into_2d(0, TESTED_PLAYBOARD_ROW_COL_SIZE), (0, 0));
        assert_eq!(i1d_into_2d(1, TESTED_PLAYBOARD_ROW_COL_SIZE), (0, 1));
        assert_eq!(i1d_into_2d(2, TESTED_PLAYBOARD_ROW_COL_SIZE), (0, 2));
        assert_eq!(i1d_into_2d(3, TESTED_PLAYBOARD_ROW_COL_SIZE), (1, 0));
        assert_eq!(i1d_into_2d(4, TESTED_PLAYBOARD_ROW_COL_SIZE), (1, 1));
        assert_eq!(i1d_into_2d(5, TESTED_PLAYBOARD_ROW_COL_SIZE), (1, 2));
        assert_eq!(i1d_into_2d(6, TESTED_PLAYBOARD_ROW_COL_SIZE), (2, 0));
        assert_eq!(i1d_into_2d(7, TESTED_PLAYBOARD_ROW_COL_SIZE), (2, 1));
        assert_eq!(i1d_into_2d(8, TESTED_PLAYBOARD_ROW_COL_SIZE), (2, 2));
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
    fn test_check_for_game_win() {
        // todo
    }

    #[test]
    fn test_invalid_place_on_grid() {
        let mut playboard = prepare_playboard();

        // Invalid place options.
        // Row out of board:
        assert_eq!(
            playboard.place_on_grid(3, 2, StartOrder::First),
            GameState::InvalidPlace
        );
        // Col out of board:
        assert_eq!(
            playboard.place_on_grid(2, 3, StartOrder::First),
            GameState::InvalidPlace
        );
        // Field is not free:
        assert_eq!(
            playboard.place_on_grid(0, 1, StartOrder::First),
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
            playboard.place_on_grid(0, 0, StartOrder::Second),
            GameState::Placed
        );
        assert_eq!(playboard.grid[0], PlayboardGridOptions::O);

        assert_eq!(
            playboard.place_on_grid(2, 1, StartOrder::First),
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
            playboard.place_on_grid(1, 1, StartOrder::Second),
            GameState::Draw
        );
        assert_eq!(playboard.grid[4], PlayboardGridOptions::O);

        // Winning in row.
        playboard.grid[1] = PlayboardGridOptions::O;
        playboard.grid[2] = PlayboardGridOptions::Free;
        assert_eq!(
            playboard.place_on_grid(0, 2, StartOrder::Second),
            GameState::GameOver
        );

        // Winning in col.
        playboard.grid[1] = PlayboardGridOptions::X;
        playboard.grid[4] = PlayboardGridOptions::Free;
        assert_eq!(
            playboard.place_on_grid(1, 1, StartOrder::First),
            GameState::GameOver
        );
    }

    #[test]
    fn test_clear_board() {
        let mut playboard = prepare_playboard();

        let expected_result = [PlayboardGridOptions::Free; TESTED_PLAYBOARD_SIZE];

        playboard.clear_board();

        assert_eq!(playboard.grid, expected_result);
    }

    #[test]
    fn test_get_row_iter() {
        let playboard = prepare_playboard();

        {
            let mut result = playboard.get_row_iter(0);

            assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
        }

        {
            let mut result = playboard.get_row_iter(1);

            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::O));
        }

        {
            let mut result = playboard.get_row_iter(2);

            assert_eq!(result.next(), Some(&PlayboardGridOptions::O));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
        }
    }

    #[test]
    fn test_get_col_iter() {
        let playboard = prepare_playboard();

        {
            let mut result = playboard.get_col_iter(0);

            assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::O));
        }

        {
            let mut result = playboard.get_col_iter(1);

            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
        }

        {
            let mut result = playboard.get_col_iter(2);

            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::O));
            assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
        }
    }
    
    #[test]
    fn test_get_main_diag_iter() {
        let playboard = prepare_playboard();

        let mut result = playboard.get_main_diag_iter();

        assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
        assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
        assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
    }

    #[test]
    fn test_get_anti_diag_iter() {
        let playboard = prepare_playboard();

        let mut result = playboard.get_anti_diag_iter();

        assert_eq!(result.next(), Some(&PlayboardGridOptions::X));
        assert_eq!(result.next(), Some(&PlayboardGridOptions::Free));
        assert_eq!(result.next(), Some(&PlayboardGridOptions::O));
    }


    // Quickcheck:
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for PlayboardGridOptions {
        fn arbitrary(g: &mut Gen) -> PlayboardGridOptions {
            g.choose(&[PlayboardGridOptions::Free, PlayboardGridOptions::X, PlayboardGridOptions::O]).unwrap().clone()
        }
    }

    impl Arbitrary for Playboard {
        fn arbitrary(g: &mut quickcheck::Gen) -> Playboard {
            let mut playboard = Playboard
            {
                grid: vec![PlayboardGridOptions::Free; TESTED_PLAYBOARD_SIZE],
                row_col_size: TESTED_PLAYBOARD_ROW_COL_SIZE,
                size: TESTED_PLAYBOARD_SIZE
            };

            for i in 0..TESTED_PLAYBOARD_SIZE {
                playboard.grid[i] = PlayboardGridOptions::arbitrary(g);
            }

            playboard
        }
    }
    
    fn check_if_same_symbols_naive(symbols: &[PlayboardGridOptions]) -> bool {
        symbols[0] == symbols[1] && symbols[0] == symbols[2] && symbols[0] != PlayboardGridOptions::Free
    }

    fn check_for_full_playboard_naive(playboard: &Playboard) -> bool {
        playboard.grid[0] != PlayboardGridOptions::Free &&
        playboard.grid[1] != PlayboardGridOptions::Free &&
        playboard.grid[2] != PlayboardGridOptions::Free &&
        playboard.grid[3] != PlayboardGridOptions::Free &&
        playboard.grid[4] != PlayboardGridOptions::Free &&
        playboard.grid[5] != PlayboardGridOptions::Free &&
        playboard.grid[6] != PlayboardGridOptions::Free &&
        playboard.grid[7] != PlayboardGridOptions::Free &&
        playboard.grid[8] != PlayboardGridOptions::Free
    }

    quickcheck::quickcheck! {
        fn test_check_if_same_symbols(playboard: Playboard) -> bool {
            let symbols = &playboard.grid[0..TESTED_PLAYBOARD_ROW_COL_SIZE];
            assert_eq!(check_if_same_symbols_naive(symbols), Playboard::check_if_same_symbols(symbols.iter()));
            true
        }

        fn test_check_for_full_playboard(playboard: Playboard) -> bool {
            assert_eq!(check_for_full_playboard_naive(&playboard), playboard.check_for_full_playboard());
            true
        }
    }
}

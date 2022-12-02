use std::io::stdout;

use crossterm::{execute, event::{KeyEvent, read, KeyCode, KeyModifiers, KeyEventKind, KeyEventState, Event}, terminal::{Clear, ClearType}, cursor::MoveTo};
use rand::random;

fn clear_terminal() {
    execute!(
        stdout(),
        Clear(ClearType::FromCursorDown),
        MoveTo(0, 0)
    ).unwrap()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Discovered(usize),
    Unseen,
    Mine
}

struct MSGrid {
    cursor: (usize, usize),
    data: Vec<Vec<Slot>>,
    num_unseen_tiles: usize
}

impl MSGrid {

    fn get(&self, (x, y): (usize, usize)) -> Option<&Slot> {
        self.data.get(y).and_then(|option| option.get(x))
    }

    fn get_mut(&mut self, (x, y): (usize, usize)) -> Option<&mut Slot> {
        self.data.get_mut(y).and_then(|option| option.get_mut(x))
    }

    pub fn new((width, height): (usize, usize), mine_density: f32) -> Self {

        let mut data = vec![vec![Slot::Unseen ; width] ; height];

        let mut num_unseen_tiles = width * height;

        for slot in data.iter_mut().flatten() {
            if random::<f32>() < mine_density {
                *slot = Slot::Mine;
                num_unseen_tiles -= 1;
            }
        }

        Self { data, cursor: (0, 0), num_unseen_tiles }
    }

    pub fn print(&self, show_mines: bool) {
        for (y, row) in self.data.iter().enumerate() {
            for (x, slot) in row.iter().enumerate() {
                let str = match slot {
                    Slot::Discovered(num_adjacent_mines) => {
                        [" ", "1", "2", "3", "4", "5", "6", "7", "8", "9"][*num_adjacent_mines]
                    }
                    Slot::Unseen => "-",
                    Slot::Mine => if show_mines {
                        "@"
                    } else {
                        "-"
                    },
                };

                let cursor = if (x, y) == self.cursor { "|" } else { " " };
                print!("{cursor}{str}{cursor}");
            }
            println!();
        }
    }

    pub fn move_cursor_up(&mut self) { self.cursor.1 = self.cursor.1.checked_sub(1).unwrap_or(self.data.len() - 1) }
    pub fn move_cursor_down(&mut self) { self.cursor.1 = (self.cursor.1 + 1) % self.data.len() }
    pub fn move_cursor_left(&mut self) { self.cursor.0 = self.cursor.0.checked_sub(1).unwrap_or(self.data.len() - 1) }
    pub fn move_cursor_right(&mut self) { self.cursor.0 = (self.cursor.0 + 1) % self.data[0].len() }

    fn num_adjacent_mines_at(&self, (i, j): (usize, usize)) -> usize {
        let mut accumulator = 0;
        for x in i - 1..=i + 1  {
            for y in j - 1..=j + 1 {
                match self.get((x, y)) {
                    Some(slot) => if *slot == Slot::Mine {
                        accumulator += 1;
                    },
                    _ => (),
                }
            }
        }
        accumulator
    }

    /// given self.data is not a mine, sweep this tile and
    /// attempt to sweep adjacent tiles with 0 mines around them
    fn sweep_at(&mut self, pos @ (x, y): (usize, usize), visited_tiles: &mut Vec<(usize, usize)>) {

        let n = self.num_adjacent_mines_at(pos);
        *self.get_mut(pos).unwrap() = Slot::Discovered(n);
        self.num_unseen_tiles -= 1;

        if n == 0 {
            for next_pos in [
                (x - 1, y),
                (x + 1, y),
                (x, y + 1),
                (x, y - 1)
            ] {

                match self.get(next_pos) {

                    Some(tile) if *tile == Slot::Unseen =>  {

                        if !visited_tiles.contains(&next_pos) {

                            visited_tiles.push(next_pos);
                            self.sweep_at(next_pos, visited_tiles);
                        }
                    },
                    _ => (),
                }
            }
        }
    }

    pub fn try_sweep_at_cursor(&mut self) -> bool {

        if *self.get(self.cursor).unwrap() == Slot::Mine {
            return false

        } else {
            let mut visited_tiles = Vec::new();
            self.sweep_at(self.cursor, &mut visited_tiles);
            return true
        }
    }
}

fn main() {
    let mut grid = MSGrid::new((20, 20), 0.1);

    let victory = loop {

        clear_terminal();
        grid.print(false);

        match read().unwrap() {

            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
                code,
            }) => match code {

                KeyCode::Char('f') => if !grid.try_sweep_at_cursor() { break 0; },
                KeyCode::Left => grid.move_cursor_left(),
                KeyCode::Right => grid.move_cursor_right(),
                KeyCode::Up => grid.move_cursor_up(),
                KeyCode::Down => grid.move_cursor_down(),

                _ => (),
            },

            _ => (),
        }
        if grid.num_unseen_tiles <= 0 { break 1; }
    };

    clear_terminal();
    grid.print(true);

    if victory == 1 {
        println!("YOU WIN NERD");
    } else if victory == 0 {
        println!("YOU LOSE NERD");
    }

    loop {}
}
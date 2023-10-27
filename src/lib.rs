use std::{
    collections::HashSet,
    fs::File,
    io::{stdout, BufRead, BufReader, Result, Stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{self},
    event::{self, poll, read, Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
};

use life_iteration::{Coords, LifeIteration};

mod life_iteration;

enum Command {
    Continue,
    Stop,
}

struct DrawingCell {
    x: u16,
    y: u16,
    r: u8,
    g: u8,
    b: u8,
}

struct DrawingCoords {
    x: u16,
    y: u16,
}

struct DrawingData {
    data: Vec<DrawingCell>,
    min_coords: DrawingCoords,
    max_coords: DrawingCoords,
}

pub fn run() -> Result<()> {
    let mut stdout = stdout();

    enable_raw_mode()?;

    execute!(stdout, cursor::Hide, terminal::EnterAlternateScreen)?;

    let drawing_data = get_drawing_data()?;
    show_help(&mut stdout, &drawing_data)?;

    loop {
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            event::EnableMouseCapture
        )?;

        let initial_state = draw_initial_state(&mut stdout, &drawing_data)?;

        execute!(stdout, event::DisableMouseCapture)?;

        if initial_state.is_none() {
            reset_win(&mut stdout)?;
            return Ok(());
        }

        let result = render_game(&mut stdout, initial_state.unwrap(), &drawing_data)?;

        if let Command::Stop = result {
            reset_win(&mut stdout)?;
            return Ok(());
        }
    }
}

fn show_help(stdout: &mut Stdout, drawing_data: &DrawingData) -> Result<()> {
    loop {
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        let y_offset = draw_peepo(stdout, drawing_data)?;

        let (width, height) = size()?;

        stdout.flush()?;

        let x_offset = (width as f32 / 2.0 - 7.0) as u16;
        let y_offset = if y_offset == 0 {
            ((height) as f32 / 2.0 - 5.0) as u16
        } else {
            y_offset + 2
        };

        execute!(
            stdout,
            cursor::MoveTo(x_offset, y_offset),
            style::Print("    Press Any Key    "),
            cursor::MoveTo(x_offset, y_offset + 1),
            style::Print("┎-------------------┒"),
            cursor::MoveTo(x_offset, y_offset + 2),
            style::Print("│ Help           F1 │"),
            cursor::MoveTo(x_offset, y_offset + 3),
            style::Print("│===================│"),
            cursor::MoveTo(x_offset, y_offset + 4),
            style::Print("│ Continue:   Enter │"),
            cursor::MoveTo(x_offset, y_offset + 5),
            style::Print("│                   │"),
            cursor::MoveTo(x_offset, y_offset + 6),
            style::Print("│ Quit:       q (Q) │"),
            cursor::MoveTo(x_offset, y_offset + 7),
            style::Print("│                   │"),
            cursor::MoveTo(x_offset, y_offset + 8),
            style::Print("│ Restart:    r (R) │"),
            cursor::MoveTo(x_offset, y_offset + 9),
            style::Print("┖-------------------┚")
        )?;

        loop {
            match read()? {
                Event::Resize(_, _) => break,
                Event::Key(KeyEvent {
                    code: _,
                    modifiers: _,
                    kind: _,
                    state: _,
                }) => {
                    return Ok(());
                }
                _ => (),
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn show_help_and_recover_screen(
    stdout: &mut Stdout,
    cells: &HashSet<Coords>,
    drawing_data: &DrawingData,
) -> Result<()> {
    show_help(stdout, drawing_data)?;

    let (win_width, win_height) = size()?;
    queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

    for cell in cells
        .iter()
        .filter(|&c| c.x >= 0 && c.y >= 0 && c.x < win_width as isize && c.y < win_height as isize)
    {
        queue!(
            stdout,
            cursor::MoveTo(cell.x as u16, cell.y as u16),
            style::Print("█")
        )?;
    }

    stdout.flush()?;
    Ok(())
}

fn reset_win(stdout: &mut Stdout) -> Result<()> {
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        event::DisableMouseCapture,
        terminal::LeaveAlternateScreen
    )?;
    disable_raw_mode()?;

    Ok(())
}

fn draw_initial_state(
    stdout: &mut Stdout,
    drawing_data: &DrawingData,
) -> Result<Option<HashSet<Coords>>> {
    let mut initial_state: HashSet<Coords> = HashSet::new();

    loop {
        match read()? {
            Event::Mouse(MouseEvent {
                kind: event,
                modifiers: _,
                row,
                column,
            }) => {
                let button_option = match event {
                    MouseEventKind::Down(button) | MouseEventKind::Drag(button) => Some(button),
                    _ => None,
                };

                if button_option.is_none() {
                    continue;
                }

                let button = button_option.unwrap();

                match button {
                    MouseButton::Left => {
                        initial_state.insert(Coords {
                            x: column as isize,
                            y: row as isize,
                        });
                        execute!(stdout, cursor::MoveTo(column, row), style::Print("█"))?;
                    }
                    MouseButton::Right => {
                        initial_state.remove(&Coords {
                            x: column as isize,
                            y: row as isize,
                        });
                        execute!(stdout, cursor::MoveTo(column, row), style::Print(" "))?;
                    }
                    _ => (),
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(key),
                modifiers: _,
                kind: _,
                state: _,
            }) if key == 'q' || key == 'Q' => {
                break Ok(None);
            }
            Event::Key(KeyEvent {
                code: KeyCode::F(1),
                modifiers: _,
                kind: _,
                state: _,
            }) => {
                show_help_and_recover_screen(stdout, &initial_state, drawing_data)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: _,
                kind: _,
                state: _,
            }) => {
                if initial_state.len() == 0 {
                    continue;
                }

                break Ok(Some(initial_state));
            }
            _ => (),
        }
    }
}

fn render_game(
    stdout: &mut Stdout,
    initial_state: HashSet<Coords>,
    drawing_data: &DrawingData,
) -> Result<Command> {
    let mut life_iteration = LifeIteration {
        cells: initial_state,
    };

    loop {
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

        let (win_width, win_height) = size()?;

        for cell in life_iteration.cells.iter().filter(|coord| {
            coord.x >= 0
                && coord.y >= 0
                && coord.x < win_width as isize
                && coord.y < win_height as isize
        }) {
            queue!(
                stdout,
                cursor::MoveTo(cell.x as u16, cell.y as u16),
                style::Print("█"),
            )?;
        }

        life_iteration = life_iteration.get_next_life_iteration();

        stdout.flush()?;

        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: _,
                    kind: _,
                    state: _,
                }) => match key {
                    k if k == 'q' || k == 'Q' => {
                        break Ok(Command::Stop);
                    }
                    k if k == 'r' || k == 'R' => break Ok(Command::Continue),
                    _ => (),
                },
                Event::Key(KeyEvent {
                    code: KeyCode::F(1),
                    modifiers: _,
                    kind: _,
                    state: _,
                }) => {
                    show_help_and_recover_screen(stdout, &life_iteration.cells, drawing_data)?;
                }
                _ => (),
            }
        }
    }
}

fn get_drawing_data() -> Result<DrawingData> {
    let file = File::open("peepo.txt")?;
    let reader = BufReader::new(file);
    let mut drawing_data: Vec<DrawingCell> = Vec::new();
    let mut min_coords = DrawingCoords { x: 0, y: 0 };
    let mut max_coords = DrawingCoords { x: 0, y: 0 };

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();

        let drawing_cell = DrawingCell {
            x: parts.next().unwrap_or("0").parse::<u16>().unwrap_or(0),
            y: parts.next().unwrap_or("0").parse::<u16>().unwrap_or(0),
            r: parts.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
            g: parts.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
            b: parts.next().unwrap_or("0").parse::<u8>().unwrap_or(0),
        };

        if drawing_cell.x == 0 || drawing_cell.y == 0 {
            continue;
        }

        if drawing_cell.x < min_coords.x {
            min_coords.x = drawing_cell.x;
        }
        if drawing_cell.y < min_coords.y {
            min_coords.y = drawing_cell.y;
        }

        if drawing_cell.x > max_coords.x {
            max_coords.x = drawing_cell.x;
        }
        if drawing_cell.y > max_coords.y {
            max_coords.y = drawing_cell.y;
        }

        drawing_data.push(drawing_cell);
    }

    Ok(DrawingData {
        data: drawing_data,
        min_coords,
        max_coords,
    })
}

fn draw_peepo(stdout: &mut Stdout, drawing_data: &DrawingData) -> Result<u16> {
    let menu_hight = 15;
    let (width, height) = size()?;

    if drawing_data.max_coords.x - drawing_data.min_coords.x < width
        && drawing_data.max_coords.y + menu_hight - drawing_data.min_coords.y < height
    {
        let drawing_width = drawing_data.max_coords.x - drawing_data.min_coords.x;
        let drawing_height = drawing_data.max_coords.y - drawing_data.min_coords.y;
        let x_offset = ((width - drawing_width - 5) as f32 / 2.0) as u16;
        let y_offset = ((height - drawing_height - menu_hight) as f32 / 2.0) as u16;

        for data in drawing_data.data.iter() {
            queue!(
                stdout,
                cursor::MoveTo(x_offset + data.x, y_offset + data.y),
                style::PrintStyledContent("█".with(Color::Rgb {
                    r: data.r,
                    g: data.g,
                    b: data.b
                }))
            )?;
        }

        return Ok(y_offset + drawing_height);
    }

    Ok(0)
}

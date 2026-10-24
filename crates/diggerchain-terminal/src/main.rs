mod agent;
mod expedition;
mod render;
mod save;
mod wallet;

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::{
    self, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
};
use diggerchain_core::{Action, Game};

const HELP: &str = "DiggerChain alpha

Usage: diggerchain [--new] [--seed N] [--save PATH] [--headless]
       diggerchain --prototype [--headless]

  --expedition  Explicitly select the default saved three-floor expedition.
  --prototype   Open the preserved authored goblin playground instead.
  --new         Archive the previous save and create a new expedition.
  --seed N      Create a repeatable practice world with a chosen u64 seed.
  --save PATH   Use this save file instead of the user data directory.
  --headless    Read line commands and emit JSON observations, without graphics.
  --glyphs      Start with top-down colored glyphs (default).
  --wallet PAGE Serve a local development wallet page, separately from alpha play.
  --help        Show this help.
  --version     Show the version.

By default, play or resume a human delver's three-floor expedition (512 actions).
Alpha play is offline and has no token payouts.
Move: arrows / WASD / HJKL. E / Enter: interact. G: view. Z / X: zoom.
Expedition: bump veins to dig; E uses a flask or portal; F finishes at the current score.
Every accepted expedition action is saved. Q exits; run again to resume.
Prototype: R restarts. Space pauses ambient motion. ?: help.
";

fn main() {
    if let Err(error) = run() {
        eprintln!("DiggerChain: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut headless = false;
    let mut glyphs = true;
    let mut expedition = false;
    let mut prototype = false;
    let mut new = false;
    let mut seed = None;
    let mut save_path = None;
    let mut wallet_page = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                print!("{HELP}");
                return Ok(());
            }
            "--version" | "-V" => {
                println!("DiggerChain {}-alpha.1", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--headless" => headless = true,
            "--glyphs" => glyphs = true,
            "--expedition" => expedition = true,
            "--prototype" => prototype = true,
            "--new" => new = true,
            "--wallet" => {
                wallet_page = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--wallet requires a public HTML page path",
                    )
                })?));
            }
            "--seed" => {
                seed = Some(
                    arguments
                        .next()
                        .and_then(|value| value.parse::<u64>().ok())
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "--seed requires an unsigned 64-bit integer",
                            )
                        })?,
                );
            }
            "--save" => {
                save_path = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "--save requires a path")
                })?));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument {argument:?}; use --help"),
                ));
            }
        }
    }
    if let Some(page) = wallet_page {
        if expedition || prototype || headless || new || seed.is_some() || save_path.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--wallet runs separately from gameplay options",
            ));
        }
        return wallet::run(&page);
    }
    if prototype && (expedition || new || seed.is_some() || save_path.is_some()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--prototype cannot be combined with --expedition, --new, --seed, or --save",
        ));
    }
    if headless && prototype {
        return agent::run(io::stdin().lock(), io::stdout().lock());
    }
    if !headless && (!io::stdin().is_terminal() || !io::stdout().is_terminal()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "interactive mode needs a terminal; use --headless for pipes",
        ));
    }
    if !prototype {
        let path = match save_path {
            Some(path) => path,
            None => save::default_path()?,
        };
        let (save, record) = save::Save::open(path, new, seed)?;
        let session = expedition::Session::new(save, record)?;
        return if headless {
            expedition::agent(session, io::stdin().lock(), io::stdout().lock())
        } else {
            expedition::play(session, glyphs)
        };
    }
    play(glyphs)
}

struct TerminalSession;

impl TerminalSession {
    fn enter() -> io::Result<Self> {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous_hook(info);
        }));
        terminal::enable_raw_mode()?;
        let session = Self;
        execute!(io::stdout(), EnterAlternateScreen, Hide, DisableLineWrap)?;
        Ok(session)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        ResetColor,
        Show,
        EnableLineWrap,
        LeaveAlternateScreen
    );
}

fn play(glyphs: bool) -> io::Result<()> {
    let _session = TerminalSession::enter()?;
    let mut game = Game::new();
    let mut view = render::View::new(glyphs);
    let mut stdout = io::stdout().lock();
    let mut last = Instant::now();
    let frame = Duration::from_millis(33);
    loop {
        let started = Instant::now();
        let dt = started.duration_since(last).as_secs_f32().min(0.1);
        last = started;
        view.tick(dt);
        let (width, height) = terminal::size()?;
        view.draw(&game, width, height, &mut stdout)?;
        let wait = frame.saturating_sub(started.elapsed());
        if !event::poll(wait)? {
            continue;
        }
        let input = event::read()?;
        let Event::Key(key) = input else { continue };
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            break;
        }
        if matches!(key.code, KeyCode::Char('q' | 'Q') | KeyCode::Esc) {
            break;
        }
        if key.code == KeyCode::Char('?') {
            view.help = !view.help;
            continue;
        }
        if view.help {
            if key.code == KeyCode::Enter {
                view.help = false;
            }
            continue;
        }
        if width < 90 || height < 30 {
            continue;
        }
        let action = match key.code {
            KeyCode::Up | KeyCode::Char('w' | 'W' | 'k' | 'K') => Some(Action::Move(0, -1)),
            KeyCode::Down | KeyCode::Char('s' | 'S' | 'j' | 'J') => Some(Action::Move(0, 1)),
            KeyCode::Left | KeyCode::Char('a' | 'A' | 'h' | 'H') => Some(Action::Move(-1, 0)),
            KeyCode::Right | KeyCode::Char('d' | 'D' | 'l' | 'L') => Some(Action::Move(1, 0)),
            KeyCode::Char('e' | 'E') | KeyCode::Enter if key.kind == KeyEventKind::Press => {
                Some(Action::Interact)
            }
            KeyCode::Char('g' | 'G') if key.kind == KeyEventKind::Press => {
                view.toggle_mode();
                None
            }
            KeyCode::Char('z' | 'Z') => {
                view.zoom(1);
                None
            }
            KeyCode::Char('x' | 'X') => {
                view.zoom(-1);
                None
            }
            KeyCode::Char(' ') if key.kind == KeyEventKind::Press => {
                view.paused = !view.paused;
                None
            }
            KeyCode::Char('r' | 'R') if key.kind == KeyEventKind::Press => {
                game = Game::new();
                view.reset_camera();
                None
            }
            _ => None,
        };
        if let Some(action) = action {
            let outcome = game.apply(action);
            view.on_event(outcome, game.player);
        }
    }
    stdout.flush()
}

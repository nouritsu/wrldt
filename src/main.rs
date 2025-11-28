use chrono::prelude::*;
use chrono_tz::Tz;
use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event};
use directories::ProjectDirs;
use ratatui::{DefaultTerminal, prelude::*};
use std::fs;
use wrldt::config::Config;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    reset_config: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let config_dir = ProjectDirs::from("com", "nouritsu", "wrldt")
        .expect("failed to get retrieve project directory")
        .config_dir()
        .to_owned();

    fs::create_dir_all(&config_dir).expect(&format!(
        "unable to create config directory at {}",
        config_dir.to_string_lossy()
    ));

    if !fs::exists(&config_dir.join("config.toml"))? || args.reset_config {
        Config::save_default(config_dir.join("config.toml"))?;
    }

    // let config = Config::parse(config_dir.join("config.toml"))?;
    let config = Config::parse(config_dir.join("config.toml"))?;
    dbg!(&config);

    let terminal = ratatui::init();
    let result = run(terminal, &config);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, config: &Config) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &config.timezones))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code.is_char('q') || key.code.is_esc() {
                    break Ok(());
                }
            }
        }
    }
}

fn render(frame: &mut Frame, tzs: &[Tz]) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage((100 / tzs.len()) as u16);
            tzs.len()
        ])
        .split(frame.area());

    for (i, tz) in tzs.iter().enumerate() {
        frame.render_widget(
            format!(
                "{}",
                Local::now().with_timezone(tz).format("%Y-%m-%d %H:%M:%S")
            ),
            layout[i],
        );
    }
}

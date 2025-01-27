use std::process::Command;
use std::env;
use crossterm::{event, execute};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use tui::{Terminal, widgets::{Block, Paragraph, Borders}, backend::CrosstermBackend, style::{Style, Color}};

use anyhow::Result;

fn main() -> Result<()> {
    enable_raw_mode()?; 
    let mut stdout = io::stdout(); 
    execute!(stdout, EnterAlternateScreen)?; 

    let prompt = "$ ";
    let mut input = String::new();

    loop {
        draw_ui(prompt, &input)?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    KeyCode::Enter => {
                        if !input.trim().is_empty() {
                            let command = input.clone();
                            match execute_command(&command) {
                                Ok(output) => println!("{}", output),
                                Err(err) => eprintln!("Error: {}", err),
                            }
                            input.clear(); 
                        }
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?; 
    execute!(io::stdout(), LeaveAlternateScreen)?; 
    Ok(())
}

fn draw_ui(prompt: &str, input: &str) -> Result<()> {
    let stdout = io::stdout(); 
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        let size = frame.size();
        let prompt_text = format!("{}{}", prompt, input);
        let paragraph = Paragraph::new(prompt_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().title("Terminal Emulator").borders(Borders::ALL));
        frame.render_widget(paragraph, size);
    })?;

    Ok(())
}

fn execute_command(command: &str) -> Result<String> {
    let mut parts = command.split_whitespace();
    let command_name = parts.next().unwrap_or("");

    let is_windows = env::consts::OS == "windows"; 

    match command_name {
        "mkdir" => {
            let dir_name = parts.next().unwrap_or("");
            if !dir_name.is_empty() {
                let output = if is_windows {
                    Command::new("cmd")
                        .arg("/C")
                        .arg(format!("mkdir {}", dir_name))
                        .output()?
                } else {
                    Command::new("mkdir").arg(dir_name).output()?
                };
                if output.status.success() {
                    Ok(format!("Directory '{}' created", dir_name))
                } else {
                    Err(anyhow::anyhow!("Failed to create directory '{}'", dir_name))
                }
            } else {
                Err(anyhow::anyhow!("Missing directory name"))
            }
        }
        "rm" => {
            let dir_name = parts.next().unwrap_or("");
            if !dir_name.is_empty() {
                let output = if is_windows {
                    Command::new("cmd")
                        .arg("/C")
                        .arg(format!("rmdir /S /Q {}", dir_name))
                        .output()?
                } else {
                    Command::new("rm").arg("-r").arg(dir_name).output()?
                };
                if output.status.success() {
                    Ok(format!("Directory '{}' removed", dir_name))
                } else {
                    Err(anyhow::anyhow!("Failed to remove directory '{}'", dir_name))
                }
            } else {
                Err(anyhow::anyhow!("Missing directory name"))
            }
        }
        "ls" => {
            let output = if is_windows {
                Command::new("cmd")
                    .arg("/C")
                    .arg("dir")
                    .output()?
            } else {
                Command::new("ls").output()?
            };
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(anyhow::anyhow!("Failed to execute 'ls'"))
            }
        }
        _ => {
            let output = if is_windows {
                Command::new("cmd")
                    .arg("/C")
                    .arg(command)
                    .output()?
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()?
            };
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(anyhow::anyhow!("Command failed: {}", command))
            }
        }
    }
}

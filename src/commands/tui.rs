use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;

use crate::config::EnvGuardianConfig;
use crate::env::parser::parse_env_file;
use crate::error::{AppError, Result};

enum Mode {
    Browse,
    Edit,
}

impl PartialEq for Mode {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Mode::Browse, Mode::Browse) | (Mode::Edit, Mode::Edit)
        )
    }
}

struct TuiApp {
    root: PathBuf,
    config: EnvGuardianConfig,
    profiles: Vec<String>,
    profile_index: usize,
    vars: Vec<(String, String)>,
    selected: usize,
    mode: Mode,
    edit_buffer: String,
    status: String,
    env_path: PathBuf,
}

impl TuiApp {
    fn new(root: PathBuf, profile: Option<&str>) -> Result<Self> {
        let config = EnvGuardianConfig::load(&root)?;
        let profiles = config.profile_names();
        let profile_name = EnvGuardianConfig::resolve_profile(profile, &config);
        let profile_index = profiles
            .iter()
            .position(|p| p == &profile_name)
            .unwrap_or(0);

        let mut app = Self {
            root,
            config,
            profiles,
            profile_index,
            vars: Vec::new(),
            selected: 0,
            mode: Mode::Browse,
            edit_buffer: String::new(),
            status: "EnvGuardian TUI — j/k navigate, Enter edit, p profile, c check, q quit"
                .to_string(),
            env_path: PathBuf::new(),
        };
        app.reload_env()?;
        Ok(app)
    }

    fn current_profile(&self) -> &str {
        &self.profiles[self.profile_index]
    }

    fn reload_env(&mut self) -> Result<()> {
        let profile = self.current_profile();
        self.env_path = self.config.env_path_for(&self.root, profile);
        if self.env_path.exists() {
            let env = parse_env_file(&self.env_path)?;
            self.vars = env.vars.into_iter().collect();
        } else {
            self.vars.clear();
            self.status = format!(
                "{} not found — press 'n' to add a key",
                self.env_path.display()
            );
        }
        if self.selected >= self.vars.len() {
            self.selected = self.vars.len().saturating_sub(1);
        }
        Ok(())
    }

    fn save_env(&mut self) -> Result<()> {
        let lines: Vec<String> = self
            .vars
            .iter()
            .map(|(k, v)| {
                if v.contains(' ') || v.contains('#') {
                    format!("{}=\"{}\"", k, v.replace('"', "\\\""))
                } else {
                    format!("{}={}", k, v)
                }
            })
            .collect();
        let content = if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n") + "\n"
        };
        std::fs::write(&self.env_path, content)?;
        self.status = format!("Saved {}", self.env_path.display());
        Ok(())
    }

    fn next_profile(&mut self) -> Result<()> {
        if self.profiles.is_empty() {
            return Ok(());
        }
        self.profile_index = (self.profile_index + 1) % self.profiles.len();
        self.reload_env()?;
        self.status = format!("Profile: {}", self.current_profile());
        Ok(())
    }

    fn run_check(&mut self) -> Result<()> {
        let profile = self.current_profile();
        let (passed, summary) =
            crate::commands::check::run_quiet(&self.root, Some(profile), false, true)?;
        self.status = if passed {
            format!("{} ({} errors)", summary, 0)
        } else {
            format!("{} — run `env-guardian check -p {}` for details", summary, profile)
        };
        Ok(())
    }

    fn start_edit(&mut self) {
        if self.vars.is_empty() {
            self.mode = Mode::Edit;
            self.edit_buffer.clear();
            self.status = "New key name:".to_string();
            return;
        }
        self.edit_buffer = self.vars[self.selected].1.clone();
        self.mode = Mode::Edit;
        self.status = format!("Edit value for {}", self.vars[self.selected].0);
    }

    fn commit_edit(&mut self) -> Result<()> {
        if self.vars.is_empty() {
            let key = self.edit_buffer.trim().to_string();
            if key.is_empty() {
                self.mode = Mode::Browse;
                return Ok(());
            }
            self.vars.push((key, String::new()));
            self.selected = 0;
            self.mode = Mode::Browse;
            self.edit_buffer.clear();
            self.status = "Key added — select and press Enter to set value".to_string();
            return Ok(());
        }

        let key = self.vars[self.selected].0.clone();
        self.vars[self.selected].1 = self.edit_buffer.clone();
        self.mode = Mode::Browse;
        self.edit_buffer.clear();
        self.save_env()?;
        self.status = format!("Updated {}", key);
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<bool> {
        match self.mode {
            Mode::Browse => match key {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.selected + 1 < self.vars.len() {
                        self.selected += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                KeyCode::Char('p') => self.next_profile()?,
                KeyCode::Char('c') => self.run_check()?,
                KeyCode::Char('n') => {
                    self.vars.push((format!("NEW_KEY_{}", self.vars.len() + 1), String::new()));
                    self.selected = self.vars.len() - 1;
                    self.start_edit();
                }
                KeyCode::Enter => self.start_edit(),
                KeyCode::Char('d') => {
                    if !self.vars.is_empty() {
                        let key = self.vars[self.selected].0.clone();
                        self.vars.remove(self.selected);
                        if self.selected >= self.vars.len() && self.selected > 0 {
                            self.selected -= 1;
                        }
                        self.save_env()?;
                        self.status = format!("Deleted {}", key);
                    }
                }
                _ => {}
            },
            Mode::Edit => match key {
                KeyCode::Esc => {
                    self.mode = Mode::Browse;
                    self.edit_buffer.clear();
                }
                KeyCode::Enter => self.commit_edit()?,
                KeyCode::Char(c) => self.edit_buffer.push(c),
                KeyCode::Backspace => {
                    self.edit_buffer.pop();
                }
                _ => {}
            },
        }
        Ok(false)
    }
}

pub fn run(root: &Path, profile: Option<&str>) -> Result<()> {
    let mut app = TuiApp::new(root.to_path_buf(), profile)?;

    enable_raw_mode().map_err(|e| AppError::Other(e.to_string()))?;
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)
        .map_err(|e| AppError::Other(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| AppError::Other(e.to_string()))?;

    let result = loop {
        terminal
            .draw(|f| draw_ui(f, &app))
            .map_err(|e| AppError::Other(e.to_string()))?;

        if event::poll(Duration::from_millis(200))
            .map_err(|e| AppError::Other(e.to_string()))?
        {
            if let Event::Key(key) = event::read().map_err(|e| AppError::Other(e.to_string()))? {
                if app.handle_key(key.code, key.modifiers)? {
                    break Ok(());
                }
            }
        }
    };

    disable_raw_mode().map_err(|e| AppError::Other(e.to_string()))?;
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .map_err(|e| AppError::Other(e.to_string()))?;

    result
}

fn draw_ui(f: &mut ratatui::Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" EnvGuardian TUI ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" | Profile: "),
            Span::styled(
                app.current_profile(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | File: "),
            Span::styled(app.env_path.display().to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(" p profile  c check  n new  d delete  Enter edit  q quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Header"));
    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = if app.vars.is_empty() {
        vec![ListItem::new("No variables — press 'n' to add")]
    } else {
        app.vars
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let display_val = if v.is_empty() {
                    "(empty)".to_string()
                } else if v.len() > 40 {
                    format!("{}…", v.chars().take(40).collect::<String>())
                } else {
                    v.clone()
                };
                let style = if i == app.selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("  {} = {}", k, display_val)).style(style)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Environment variables"),
    );
    f.render_widget(list, chunks[1]);

    let status_text = if app.mode == Mode::Edit {
        format!("EDIT: {} | {}", app.status, app.edit_buffer)
    } else {
        app.status.clone()
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[2]);

    if app.mode == Mode::Edit {
        let area = centered_rect(60, 20, f.area());
        let edit = Paragraph::new(app.edit_buffer.clone())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Edit value (Enter save, Esc cancel)"),
            );
        f.render_widget(edit, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

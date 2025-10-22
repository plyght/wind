use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

use super::{
    commands::{Command, CommandRegistry},
    config::Config as TuiConfig,
    event::{Event, EventHandler},
    state::AppState,
    ui::render,
};

use crate::Repository;

pub struct App<'a> {
    config: TuiConfig,
    state: AppState<'a>,
    registry: CommandRegistry,
    should_quit: bool,
}

impl<'a> App<'a> {
    pub async fn new(config: TuiConfig, repo: &'a Repository) -> Result<Self> {
        let registry = CommandRegistry::new(&config);
        let state = AppState::new(repo).await?;

        Ok(Self {
            config,
            state,
            registry,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let (tx, mut rx) = mpsc::channel::<Event>(100);
        let event_handler = EventHandler::new(tx.clone());
        tokio::spawn(async move {
            event_handler.run().await;
        });

        self.state.spawn_background_tasks(tx.clone());

        loop {
            terminal.draw(|f| render(f, &self.state, &self.config))?;

            if let Some(event) = rx.recv().await {
                match event {
                    Event::Key(key_event) => {
                        if let Some(command) = self.registry.resolve_key(key_event, &self.state) {
                            self.execute_command(command, &tx).await?;
                        }
                    }
                    Event::Resize(width, height) => {
                        self.state.on_resize(width, height);
                    }
                    Event::BackgroundTaskComplete(result) => {
                        self.state.handle_task_result(result);
                    }
                    Event::Tick => {
                        self.state.on_tick();
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    async fn execute_command(&mut self, command: Command, tx: &mpsc::Sender<Event>) -> Result<()> {
        match command {
            Command::Quit => {
                self.should_quit = true;
            }
            Command::NextPane => {
                self.state.next_pane();
            }
            Command::PrevPane => {
                self.state.prev_pane();
            }
            Command::MoveUp => {
                self.state.move_selection(-1);
            }
            Command::MoveDown => {
                self.state.move_selection(1);
            }
            Command::MoveLeft => {
                self.state.move_horizontal(-1);
            }
            Command::MoveRight => {
                self.state.move_horizontal(1);
            }
            Command::PageUp => {
                self.state.move_selection(-10);
            }
            Command::PageDown => {
                self.state.move_selection(10);
            }
            Command::ToggleStage => {
                self.state.toggle_stage().await?;
            }
            Command::StageAll => {
                self.state.stage_all().await?;
            }
            Command::UnstageAll => {
                self.state.unstage_all().await?;
            }
            Command::Commit => {
                self.state.open_commit_editor();
            }
            Command::CommitConfirm => {
                self.state.commit().await?;
            }
            Command::CommitCancel => {
                self.state.close_commit_editor();
            }
            Command::ShowBranches => {
                self.state.show_branches();
            }
            Command::ShowDiff => {
                self.state.show_diff();
            }
            Command::ToggleCommandPalette => {
                self.state.toggle_command_palette();
            }
            Command::Refresh => {
                self.state.refresh(tx.clone());
            }
            Command::TextInput(c) => {
                self.state.handle_text_input(c);
            }
            Command::TextBackspace => {
                self.state.handle_backspace();
            }
            Command::TextDelete => {
                self.state.handle_delete();
            }
            Command::TextEnter => {
                self.state.handle_enter();
            }
        }
        Ok(())
    }
}

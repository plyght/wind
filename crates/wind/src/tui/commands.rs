use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

use super::{config::Config, state::AppState};

#[derive(Debug, Clone)]
pub enum Command {
    Quit,
    NextPane,
    PrevPane,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    ToggleStage,
    StageAll,
    UnstageAll,
    Commit,
    CommitConfirm,
    CommitCancel,
    ShowBranches,
    ShowDiff,
    ToggleCommandPalette,
    Refresh,
    TextInput(char),
    TextBackspace,
    TextDelete,
    TextEnter,
}

pub struct CommandRegistry {
    keymap: HashMap<(KeyCode, KeyModifiers), Command>,
}

impl CommandRegistry {
    pub fn new(config: &Config) -> Self {
        let mut keymap = HashMap::new();

        for binding in &config.keybindings {
            keymap.insert((binding.key, binding.modifiers), binding.command.clone());
        }

        Self { keymap }
    }

    pub fn resolve_key<'a>(&self, key_event: KeyEvent, state: &AppState<'a>) -> Option<Command> {
        if state.is_text_input_mode() {
            return match key_event.code {
                KeyCode::Char(c) if key_event.modifiers.is_empty() => Some(Command::TextInput(c)),
                KeyCode::Backspace => Some(Command::TextBackspace),
                KeyCode::Delete => Some(Command::TextDelete),
                KeyCode::Enter if !state.is_commit_editor() => Some(Command::TextEnter),
                KeyCode::Esc => Some(Command::CommitCancel),
                _ => self
                    .keymap
                    .get(&(key_event.code, key_event.modifiers))
                    .cloned(),
            };
        }

        self.keymap
            .get(&(key_event.code, key_event.modifiers))
            .cloned()
    }
}

impl Config {
    pub fn default_keybindings() -> Vec<KeyBinding> {
        vec![
            KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE, Command::Quit),
            KeyBinding::new(KeyCode::Char('c'), KeyModifiers::CONTROL, Command::Quit),
            KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE, Command::NextPane),
            KeyBinding::new(KeyCode::BackTab, KeyModifiers::SHIFT, Command::PrevPane),
            KeyBinding::new(KeyCode::Char('k'), KeyModifiers::NONE, Command::MoveUp),
            KeyBinding::new(KeyCode::Char('j'), KeyModifiers::NONE, Command::MoveDown),
            KeyBinding::new(KeyCode::Char('h'), KeyModifiers::NONE, Command::MoveLeft),
            KeyBinding::new(KeyCode::Char('l'), KeyModifiers::NONE, Command::MoveRight),
            KeyBinding::new(KeyCode::Up, KeyModifiers::NONE, Command::MoveUp),
            KeyBinding::new(KeyCode::Down, KeyModifiers::NONE, Command::MoveDown),
            KeyBinding::new(KeyCode::Left, KeyModifiers::NONE, Command::MoveLeft),
            KeyBinding::new(KeyCode::Right, KeyModifiers::NONE, Command::MoveRight),
            KeyBinding::new(KeyCode::Char('u'), KeyModifiers::CONTROL, Command::PageUp),
            KeyBinding::new(KeyCode::Char('d'), KeyModifiers::CONTROL, Command::PageDown),
            KeyBinding::new(KeyCode::Char(' '), KeyModifiers::NONE, Command::ToggleStage),
            KeyBinding::new(KeyCode::Char('a'), KeyModifiers::NONE, Command::StageAll),
            KeyBinding::new(KeyCode::Char('u'), KeyModifiers::NONE, Command::UnstageAll),
            KeyBinding::new(KeyCode::Char('c'), KeyModifiers::NONE, Command::Commit),
            KeyBinding::new(
                KeyCode::Enter,
                KeyModifiers::CONTROL,
                Command::CommitConfirm,
            ),
            KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE, Command::CommitCancel),
            KeyBinding::new(
                KeyCode::Char('b'),
                KeyModifiers::NONE,
                Command::ShowBranches,
            ),
            KeyBinding::new(KeyCode::Char('d'), KeyModifiers::NONE, Command::ShowDiff),
            KeyBinding::new(
                KeyCode::Char('p'),
                KeyModifiers::CONTROL,
                Command::ToggleCommandPalette,
            ),
            KeyBinding::new(KeyCode::Char('r'), KeyModifiers::NONE, Command::Refresh),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub command: Command,
}

impl KeyBinding {
    pub fn new(key: KeyCode, modifiers: KeyModifiers, command: Command) -> Self {
        Self {
            key,
            modifiers,
            command,
        }
    }
}

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    BackgroundTaskComplete(TaskResult),
    Tick,
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    StatusRefreshed,
    DiffLoaded(String),
    BranchesLoaded(Vec<String>),
    Error(String),
}

pub struct EventHandler {
    tx: mpsc::Sender<Event>,
}

impl EventHandler {
    pub fn new(tx: mpsc::Sender<Event>) -> Self {
        Self { tx }
    }

    pub async fn run(self) {
        let mut tick_interval = tokio::time::interval(tokio::time::Duration::from_millis(250));

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    let _ = self.tx.send(Event::Tick).await;
                }
                _ = tokio::task::spawn_blocking(|| {
                    event::read()
                }) => {
                    if let Ok(Ok(evt)) = tokio::task::spawn_blocking(event::read).await {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if self.tx.send(Event::Key(key)).await.is_err() {
                                    return;
                                }
                            }
                            CrosstermEvent::Resize(w, h) => {
                                let _ = self.tx.send(Event::Resize(w, h)).await;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

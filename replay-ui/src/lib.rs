//! replay-ui: a ratatui UI for stepping through a recorded agent session.

use anyhow::Result;
use agent_loop::Step;

/// Render the current step plus the cursor position.
pub fn render(frame: &mut ratatui::Frame, steps: &[Step], cursor: usize) {
    let area = frame.area();
    let line = steps
        .get(cursor)
        .map(|s| format!("[{}/{}] phase={:?}", cursor + 1, steps.len(), s.phase))
        .unwrap_or_else(|| "(no step)".into());
    frame.render_widget(
        ratatui::widgets::Paragraph::new(line).alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

/// Run the replay UI until the user quits (q) or reaches the end.
pub async fn run(steps: Vec<Step>) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut cursor = 0usize;
    loop {
        terminal.draw(|f| render(f, &steps, cursor))?;
        if let ratatui::crossterm::event::Event::Key(key) = ratatui::crossterm::event::read()? {
            match key.code {
                ratatui::crossterm::event::KeyCode::Char('q') => break,
                ratatui::crossterm::event::KeyCode::Right => {
                    cursor = (cursor + 1).min(steps.len().saturating_sub(1))
                }
                ratatui::crossterm::event::KeyCode::Left => cursor = cursor.saturating_sub(1),
                _ => {}
            }
        }
    }
    ratatui::restore();
    Ok(())
}

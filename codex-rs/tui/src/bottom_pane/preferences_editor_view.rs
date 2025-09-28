use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::Widget;

use super::CancellationEvent;
use super::bottom_pane_view::BottomPaneView;
use super::textarea::TextArea;
use super::textarea::TextAreaState;

pub(crate) struct PreferencesEditorView {
    path: PathBuf,
    display_path: String,
    textarea: TextArea,
    textarea_state: RefCell<TextAreaState>,
    last_saved_text: String,
    dirty: bool,
    complete: bool,
    status_message: Option<StatusMessage>,
    confirm_discard: bool,
}

struct StatusMessage {
    text: String,
    kind: StatusKind,
}

enum StatusKind {
    Info,
    Error,
    Warning,
}

impl PreferencesEditorView {
    pub(crate) fn new(path: PathBuf, contents: String) -> Self {
        let mut textarea = TextArea::new();
        textarea.set_text(&contents);
        textarea.set_cursor(textarea.text().len());
        Self {
            display_path: path.display().to_string(),
            path,
            textarea,
            textarea_state: RefCell::new(TextAreaState::default()),
            last_saved_text: contents,
            dirty: false,
            complete: false,
            status_message: None,
            confirm_discard: false,
        }
    }

    fn apply_editor_change<F: FnOnce(&mut TextArea)>(&mut self, edit: F) -> bool {
        let before = self.textarea.text().to_string();
        edit(&mut self.textarea);
        let changed = self.textarea.text() != before;
        if changed {
            self.dirty = self.textarea.text() != self.last_saved_text;
            self.status_message = None;
            self.confirm_discard = false;
        }
        changed
    }

    fn save(&mut self) {
        if let Some(parent) = self.path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                self.status_message = Some(StatusMessage::error(format!(
                    "Failed to save preferences: {err}"
                )));
                return;
            }
        }

        match fs::write(&self.path, self.textarea.text()) {
            Ok(()) => {
                self.last_saved_text = self.textarea.text().to_string();
                self.dirty = false;
                self.status_message = Some(StatusMessage::info(format!(
                    "Saved to {}",
                    self.display_path
                )));
                self.confirm_discard = false;
            }
            Err(err) => {
                self.status_message = Some(StatusMessage::error(format!(
                    "Failed to save preferences: {err}"
                )));
            }
        }
    }

    fn request_close(&mut self) {
        if self.dirty && !self.confirm_discard {
            self.confirm_discard = true;
            self.status_message = Some(StatusMessage::warning(
                "Unsaved changes. Press Esc again to discard, or Ctrl+S to save.".to_string(),
            ));
        } else {
            self.complete = true;
        }
    }

    fn status_span(&self) -> Span<'static> {
        if let Some(message) = &self.status_message {
            return message.as_span();
        }

        if self.dirty {
            "Unsaved changes — press Ctrl+S to save"
                .to_string()
                .yellow()
        } else {
            "All changes saved".to_string().green()
        }
    }

    fn input_height(&self, width: u16) -> u16 {
        let usable_width = width.saturating_sub(2);
        let text_height = self.textarea.desired_height(usable_width).clamp(4, 18);
        text_height.saturating_add(1)
    }

    fn textarea_rect(&self, area: Rect) -> Option<Rect> {
        if area.width < 4 {
            return None;
        }
        let text_area_height = self.input_height(area.width).saturating_sub(1);
        if text_area_height == 0 {
            return None;
        }
        Some(Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(4),
            width: area.width.saturating_sub(2),
            height: text_area_height,
        })
    }
}

impl BottomPaneView for PreferencesEditorView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let modifiers = key_event.modifiers;
        if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER) {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.save();
                    return;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.request_close();
                    return;
                }
                _ => {}
            }
        }

        self.apply_editor_change(|ta| ta.input(key_event));
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.request_close();
        CancellationEvent::Handled
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.input_height(width).saturating_add(5)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Title
        let mut y = area.y;
        Paragraph::new(Line::from(vec![gutter(), "Edit preferences.md".bold()])).render(
            Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            },
            buf,
        );
        y = y.saturating_add(1);

        // Path line
        Paragraph::new(Line::from(vec![
            gutter(),
            format!("Path: {}", self.display_path).dim(),
        ]))
        .render(
            Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            },
            buf,
        );
        y = y.saturating_add(1);

        // Status line
        Paragraph::new(Line::from(vec![gutter(), self.status_span()])).render(
            Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            },
            buf,
        );
        y = y.saturating_add(1);

        // Editor area with gutter
        let input_height = self.input_height(area.width);
        let input_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: input_height,
        };
        if input_area.width >= 2 {
            for row in 0..input_area.height {
                Paragraph::new(Line::from(vec![gutter()])).render(
                    Rect {
                        x: input_area.x,
                        y: input_area.y.saturating_add(row),
                        width: 2,
                        height: 1,
                    },
                    buf,
                );
            }

            let text_area_height = input_area.height.saturating_sub(1);
            if text_area_height > 0 {
                if input_area.width > 2 {
                    Clear.render(
                        Rect {
                            x: input_area.x.saturating_add(2),
                            y: input_area.y,
                            width: input_area.width.saturating_sub(2),
                            height: 1,
                        },
                        buf,
                    );
                }
                if let Some(rect) = self.textarea_rect(area) {
                    let mut state = self.textarea_state.borrow_mut();
                    StatefulWidgetRef::render_ref(&(&self.textarea), rect, buf, &mut state);
                    if self.textarea.text().is_empty() {
                        Paragraph::new(Line::from(vec![
                            "Type your preferences and press Ctrl+S to save".dim(),
                        ]))
                        .render(rect, buf);
                    }
                }
            }
        }
        y = y.saturating_add(input_area.height);

        // Blank spacer before hint
        if y < area.y.saturating_add(area.height) {
            Clear.render(
                Rect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: 1,
                },
                buf,
            );
        }

        let hint_y = y.saturating_add(1);
        if hint_y < area.y.saturating_add(area.height) {
            Paragraph::new(Line::from(vec![
                gutter(),
                "Ctrl+S save · Esc close".to_string().dim(),
            ]))
            .render(
                Rect {
                    x: area.x,
                    y: hint_y,
                    width: area.width,
                    height: 1,
                },
                buf,
            );
        }
    }

    fn handle_paste(&mut self, pasted: String) -> bool {
        self.apply_editor_change(|ta| ta.insert_str(&pasted))
    }

    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        let rect = self.textarea_rect(area)?;
        let state = *self.textarea_state.borrow();
        self.textarea.cursor_pos_with_state(rect, state)
    }
}

impl StatusMessage {
    fn info(text: String) -> Self {
        Self {
            text,
            kind: StatusKind::Info,
        }
    }

    fn error(text: String) -> Self {
        Self {
            text,
            kind: StatusKind::Error,
        }
    }

    fn warning(text: String) -> Self {
        Self {
            text,
            kind: StatusKind::Warning,
        }
    }

    fn as_span(&self) -> Span<'static> {
        match self.kind {
            StatusKind::Info => self.text.clone().green(),
            StatusKind::Error => self.text.clone().red(),
            StatusKind::Warning => self.text.clone().yellow(),
        }
    }
}

fn gutter() -> Span<'static> {
    "▌ ".cyan()
}

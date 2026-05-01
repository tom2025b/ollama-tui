use super::App;

impl App {
    /// Move the chat history view upward by the given number of lines.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    /// Move the chat history view downward (toward newest messages).
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Jump to the very top of the chat history.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = usize::MAX;
    }

    /// Jump back to the newest messages at the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Show or hide the help overlay.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.status = if self.show_help {
            "Help is open. Press q, Esc, ?, or Ctrl-C to close it.".to_string()
        } else {
            "Help closed.".to_string()
        };
    }

    /// Hide the help overlay.
    pub fn hide_help(&mut self) {
        self.show_help = false;
        self.status = "Help closed.".to_string();
    }

    /// Mark the app as ready to close.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

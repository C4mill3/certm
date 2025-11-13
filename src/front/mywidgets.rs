use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, prelude::*, widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, WidgetRef, Wrap}
};

use crate::tools::{self};
use tools::certs_manager::{Realm, Cert, KeySize, CertType};

// Custom widget to render a Paragraph with a scrollbar
pub struct ScrollableParagraph {
    text: String,
    scroll: usize,
    max_scroll: usize,
}

impl ScrollableParagraph {
    pub fn new(text: String, scroll: usize, max_scroll: usize) -> Self {
        Self { text, scroll, max_scroll }
    }
}

impl WidgetRef for ScrollableParagraph {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let content_height = self.text.lines().count() as usize;  // Simplified; adjust for wrapping if needed
        let visible_height = area.height as usize;

        if content_height > visible_height {
            // Split for text and scrollbar
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(area);

            let text_area = chunks[0];
            let scrollbar_area = chunks[1];

            let paragraph = Paragraph::new(self.text.clone())
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .scroll((self.scroll as u16, 0));

            paragraph.render(text_area, buf);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut state = ScrollbarState::new(self.max_scroll+1).position(self.scroll);
            scrollbar.render(scrollbar_area, buf, &mut state);
        } else {
            // No scrollbar needed
            let paragraph = Paragraph::new(self.text.clone())
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            paragraph.render(area, buf);
        }
    }
}

/// Only needed for backwards compatibility
impl Widget for ScrollableParagraph {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}



//////////////////////////////

// Custom widget to render a Form for creating a new cert
pub struct NewCertForm {
    cursor: usize,
    newcert_type: CertType,
    newcert_keysize: KeySize,
    newcert_cn: String,
    newcert_altdns: Vec<String>,
    newcert_altip: Vec<String>,
}

impl NewCertForm {
    pub fn new(cursor: usize, newcert_type: CertType, newcert_keysize: KeySize, newcert_cn: String,
                newcert_altdns: Vec<String>, newcert_altip: Vec<String>) -> Self {
        Self{cursor, newcert_type, newcert_keysize, newcert_cn, newcert_altdns, newcert_altip}
    }
}

impl WidgetRef for NewCertForm {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Realm block
            Constraint::Length(10), // CA block
            Constraint::Length(2), // Buttons
        ]).split(area);

        // x.render(chunks, buf)
    }
}

/// Only needed for backwards compatibility
impl Widget for NewCertForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}
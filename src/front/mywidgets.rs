use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, prelude::*, widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, WidgetRef, Wrap}
};

use crate::tools::{self};
use tools::certs_manager::{Realm, Cert, KeySize, CertType};

use super::ui::{format_with_ellipsis, format_date};

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
    is_on_content: bool,
    cursor: usize, // Common Name: 0 / SAN (DNS): 1 / SAN IP (Optional): 2 / Certificat Type: 3/ Certificate Keysize: 4
    newcert_type: usize,
    newcert_keysize: usize,
    newcert_cn: String,
    newcert_altdns: String,
    newcert_altip: String,
    new_cert_validuntil: String,
}

impl NewCertForm {
    pub fn new(is_on_content: bool, cursor: usize, newcert_type: usize, newcert_keysize: usize, newcert_cn: &String,
                newcert_altdns: &String, newcert_altip: &String, new_cert_validuntil: &String) -> Self {
        Self{is_on_content, cursor, newcert_type, newcert_keysize, newcert_cn: newcert_cn.clone(), newcert_altdns: newcert_altdns.clone(), newcert_altip: newcert_altip.clone(), new_cert_validuntil: new_cert_validuntil.clone()}
    }
}

impl WidgetRef for NewCertForm {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11), // Forms Area
            Constraint::Length(1), // Buttons
        ]).split(area);
        
        let forms_area = chunks[0];
        let buttons_area = chunks[1];

        let available_width =  forms_area.width as usize;

        // Common Name: 0 / SAN (DNS): 1 / SAN IP (Optional): 2 / Certificat Type: 3/ Certificate Keysize: 4

        let common_name_full = if self.cursor == 0 && self.is_on_content { self.newcert_cn.clone() + "_" } else { self.newcert_cn.clone() };
        let altdns_full = if self.cursor == 1 && self.is_on_content { self.newcert_altdns.clone() + "_" } else { self.newcert_altdns.clone() };
        let altip_full = if self.cursor == 2 && self.is_on_content { self.newcert_altip.clone() + "_" } else { self.newcert_altip.clone() };

        let formatted_date = format_date(&self.new_cert_validuntil.clone());
        let valid_until_full = if self.cursor == 3 { formatted_date + "_" } else { formatted_date };

        let cert_type = ["Server", "Client", "Server & Client"];
        let key_sizes = [1024, 2048, 4096];
        let forms_lines = vec![
            Line::from(format_with_ellipsis("Common Name: ", &common_name_full, available_width)).style(if self.cursor == 0 && self.is_on_content { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
            Line::from(""),
            Line::from(format_with_ellipsis("SAN Domain (separate using ,): ", &altdns_full, available_width)).style(if self.cursor == 1 && self.is_on_content { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
            Line::from(""),
            Line::from(format_with_ellipsis("SAN IP (separate using ,): ", &altip_full, available_width)).style(if self.cursor == 2 && self.is_on_content { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
            Line::from(""),
            Line::from(format_with_ellipsis("Valid Until (DD/MM/YYYY): ", &valid_until_full, available_width)).style(if self.cursor == 3 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
            Line::from(""),
            Line::from(format!("Certificate Type: {}{}{}",
                if self.cursor == 4 && self.newcert_type > 0 { "← " } else { "  " },
                cert_type[self.newcert_type],
                if self.cursor == 4 && self.newcert_type < cert_type.len().saturating_sub(1) { " →" } else { "" })).style(if self.cursor == 4 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
            Line::from(""),
            Line::from(format!("Key Size: {} {} {}",
                if self.cursor == 5 && self.newcert_keysize > 0 { "←" } else { " " },
                key_sizes[self.newcert_keysize],
                if self.cursor == 5 && self.newcert_keysize < key_sizes.len().saturating_sub(1) { "→" } else { " " })).style(if self.cursor == 5 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() }),
        ];
        let forms_paragraph = Paragraph::new(forms_lines).wrap(Wrap { trim: true });
        forms_paragraph.render(forms_area, buf);

//        // Buttons

        let create_text = if self.cursor == 6 { "[Create]" } else { " Create " };
        let create_paragraph = Paragraph::new(create_text)
            .alignment(Alignment::Center)
            .style(if self.cursor == 6 { Style::default().add_modifier(Modifier::BOLD) } else { Style::default() });
        create_paragraph.render(buttons_area, buf);
    }
}

/// Only needed for backwards compatibility
impl Widget for NewCertForm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}
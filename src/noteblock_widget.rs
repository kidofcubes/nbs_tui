use tui::{widgets::{Borders, BorderType, Widget}, style::{Style, Modifier, Color}, layout::Rect, buffer::Buffer};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteblockWidget {
    /// Visible borders
    borders: Borders,
    /// Border style
    border_style: Style,
    /// Type of the border. The default is plain lines but one can choose to have rounded corners
    /// or doubled lines instead.
    border_type: BorderType,
    /// Widget style
    style: Style,
    block_width: u16,
    block_height: u16,
}

impl Default for NoteblockWidget {
    fn default() -> NoteblockWidget {
        NoteblockWidget {
            borders: Borders::NONE,
            border_style: Default::default(),
            border_type: BorderType::Plain,
            style: Default::default(),
            block_width: 4,
            block_height: 2,
        }
    }
}

impl NoteblockWidget {

    pub fn border_style(mut self, style: Style) -> NoteblockWidget {
        self.border_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> NoteblockWidget {
        self.style = style;
        self
    }

    pub fn borders(mut self, flag: Borders) -> NoteblockWidget {
        self.borders = flag;
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> NoteblockWidget {
        self.border_type = border_type;
        self
    }

    /// Compute the inner area of a block based on its border visibility rules.
    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;
        if self.borders.intersects(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1).min(inner.right());
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::TOP) {
            inner.y = inner.y.saturating_add(1).min(inner.bottom());
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::BOTTOM) {
            inner.height = inner.height.saturating_sub(1);
        }
        inner
    }
}

impl Widget for NoteblockWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }
        buf.set_style(area, self.style);
        let symbols = BorderType::line_symbols(self.border_type);
        
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if x%self.block_width == 0 && y%self.block_height == 0 {
                    buf.get_mut(x,y)
                    .set_symbol(symbols.cross)
                    .set_style(self.border_style);
                    continue;
                }
                if x%self.block_width == 0 {
                    buf.get_mut(x,y)
                    .set_symbol(symbols.vertical)
                    .set_style(self.border_style);
                    continue;
                }
                if y%self.block_height == 0 {
                    buf.get_mut(x,y)
                    .set_symbol(symbols.horizontal)
                    .set_style(self.border_style);
                    continue;
                }
                if x%self.block_width == 1 {
                    buf.get_mut(x,y)
                    .set_symbol("2")
                    .set_style(Style::default().fg(Color::Green));
                }
                if x%self.block_width == 2 {
                    buf.get_mut(x,y)
                    .set_symbol("0")
                    .set_style(Style::default().fg(Color::Green));
                }
                if x%self.block_width == 3 {
                    buf.get_mut(x,y)
                    .set_symbol("D")
                    .set_style(Style::default().fg(Color::DarkGray));
                }
                
            }
        }
    }
}
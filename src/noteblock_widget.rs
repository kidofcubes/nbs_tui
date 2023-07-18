use ratatui::{widgets::{StatefulWidget}, style::{Style, Color}, layout::Rect, buffer::{Buffer, Cell}};

use crate::{editor::EditorState, parsers::{NoteblockSection}};
#[derive(Debug)]
pub struct NoteblockWidget {
    /// Type of the border. The default is plain lines but one can choose to have rounded corners
    /// or doubled lines instead.
    pub block_width: u16,
    pub block_height: u16
}


const INSTRUMENT_COLORS: [ratatui::style::Color; 16] = [
    // Color::Rgb(0, 70, 140),
    // Color::Rgb(34, 110, 45),
    // Color::Rgb(156, 77, 79),
    // Color::Rgb(156, 156, 0),
    // Color::Rgb(124, 62, 122),
    // Color::Rgb(103, 45, 31),
    // Color::Rgb(156, 149, 63),
    // Color::Rgb(156, 0, 156),
    // Color::Rgb(54, 111, 124),
    // Color::Rgb(156, 156, 156),
    // Color::Rgb(0, 112, 156),
    // Color::Rgb(156, 10, 14),
    // Color::Rgb(156, 59, 15),
    // Color::Rgb(0, 156, 0),
    // Color::Rgb(156, 0, 59),
    // Color::Rgb(60, 60, 60),


    Color::Rgb(25, 100, 172), //official colors
    Color::Rgb(60, 142, 72),
    Color::Rgb(190, 107, 107),
    Color::Rgb(190, 190, 25),
    Color::Rgb(157, 90, 152),
    Color::Rgb(77, 60, 152),
    Color::Rgb(190, 182, 92),
    Color::Rgb(190, 25, 190),
    Color::Rgb(82, 142, 157),
    Color::Rgb(190, 190, 190),
    Color::Rgb(25, 145, 190),
    Color::Rgb(190, 35, 40),
    Color::Rgb(190, 87, 40),
    Color::Rgb(25, 190, 25),
    Color::Rgb(190, 25, 87),
    Color::Rgb(87, 87, 87),
    ];

fn get_instrument_color(index : i8) -> Color{
    if index >= INSTRUMENT_COLORS.len() as i8 {
        return Color::Rgb(255,0,0);
    }
    return INSTRUMENT_COLORS[index as usize];
}

impl StatefulWidget for NoteblockWidget {
    type State = EditorState;

    fn render(self, area: Rect, buf: &mut Buffer, editor_state: &mut EditorState) {
        if area.area() == 0 {
            return;
        }
        // buf.set_style(area, self.style);
        let inner_style = Style::default().fg(Color::White);

        let mut tick: i32 = editor_state.prev_tick as i32;
        let mut layer: u16 = 0;
        
        for index in editor_state.prev_index as usize ..editor_state.song.as_mut().unwrap().noteblocks.len(){
            match &editor_state.song.as_mut().unwrap().noteblocks[index]{
                NoteblockSection::SetTick(num) => {
                    tick = *num as i32;
                    // if ((tick as f32 - editor_state.tick) * self.block_width as f32) as u16 >= area.right() {
                    //     break;
                    // }
                },
                NoteblockSection::SetLayer(num) => {
                    layer = *num as u16;
                },
                NoteblockSection::Noteblock(noteblock) => {
                    if (layer as u16+1) * self.block_height <= area.bottom() {
                        let real_x: u16 = ((tick as f32-editor_state.tick)*self.block_width as f32).floor() as u16;
                        let real_y: u16 = layer *self.block_height;
                        if real_y+self.block_height+1 >=area.bottom() || 
                            real_y < area.top() ||
                            real_x+self.block_width+1 >= area.right() ||
                            real_x < area.left() 
                        {
                            continue;
                        }

                        let border_style = Style::default().fg(get_instrument_color(noteblock.instrument));
            
                        for num in 1..self.block_width {
                            add_to_cell(buf,real_x+num,real_y,HORI,&border_style);
                            add_to_cell(buf,real_x+num,real_y+self.block_height,HORI,&border_style);
                        }
                        for num in 1..self.block_height {
                            add_to_cell(buf,real_x,real_y+num,VERT,&border_style);
                            add_to_cell(buf,real_x+self.block_width,real_y+num,VERT,&border_style);
                        }
            
                        
                        add_to_cell(buf,real_x,real_y,RIGHT_DOWN,&border_style);
                        add_to_cell(buf,real_x+self.block_width,real_y,LEFT_DOWN,&border_style);
                        add_to_cell(buf,real_x,real_y+self.block_height,RIGHT_UP,&border_style);
                        add_to_cell(buf,real_x+self.block_width,real_y+self.block_height,LEFT_UP,&border_style);
            
                        let mut key : String;
                        if noteblock.key-33 < 0 {
                            key = "<".to_string();
                        } else if noteblock.key-33 > 57{
                            key = ">".to_string();
                        } else {
                            key = (noteblock.key-33).to_string();
                        }
                        if key.len() == 1 {
                            key = format!(" {}", key);
                        }
                        
            
                        
                        buf.get_mut(real_x+1,real_y+1)
                        .set_char(key.chars().nth(0).unwrap())
                        .set_style(inner_style);
                        buf.get_mut(real_x+2,real_y+1)
                        .set_char(key.chars().nth(1).unwrap())
                        .set_style(inner_style);
                        buf.get_mut(real_x+3,real_y+1)
                        .set_symbol(" ")
                        .set_style(inner_style);
                    }
                },
            }
        }

        // for block in self.blocks {

        // }
    }


}


const VERT_STR: &str = "│";
const HORI_STR: &str = "─";
// const LEFT_DOWN_STR: &str = "┐";
const LEFT_DOWN_STR: &str = "╮";
// const RIGHT_DOWN_STR: &str = "┌";
const RIGHT_DOWN_STR: &str = "╭";
// const LEFT_UP_STR: &str = "┘";
const LEFT_UP_STR: &str = "╯";
// const RIGHT_UP_STR: &str = "└";
const RIGHT_UP_STR: &str = "╰";
const VERT_LEFT_STR: &str = "┤";
const VERT_RIGHT_STR: &str = "├";
const HORI_DOWN_STR: &str = "┬";
const HORI_UP_STR: &str = "┴";
const CROSS_STR: &str = "┼";

const LEFT_DOWN : u8 = 0b00000011;
const RIGHT_DOWN : u8 = 0b00001001;
const LEFT_UP : u8 = 0b00000110;
const RIGHT_UP : u8 = 0b00001100;

const HORI_UP : u8 = 0b00001110;
const HORI_DOWN : u8 = 0b00001011;
const VERT_RIGHT : u8 = 0b00001101;
const VERT_LEFT : u8 = 0b00000111;

const HORI : u8 = 0b00001010;
const VERT : u8 = 0b00000101;

const CROSS : u8 = 0b00001111;

const BORDERS: [&str; 16] = [
    " ",
    " ",
    " ",
    LEFT_DOWN_STR,
    " ",
    VERT_STR,
    LEFT_UP_STR,
    VERT_LEFT_STR,
    " ",
    RIGHT_DOWN_STR,
    HORI_STR,
    HORI_DOWN_STR,
    RIGHT_UP_STR,
    VERT_RIGHT_STR,
    HORI_UP_STR,
    CROSS_STR,
    ];
fn char_to_border(str: &str) -> u8{
    match str{
        LEFT_DOWN_STR => 3,

        VERT_STR => 5,
        LEFT_UP_STR => 6,
        VERT_LEFT_STR => 7,

        RIGHT_DOWN_STR => 9,
        HORI_STR => 10,
        HORI_DOWN_STR => 11,
        RIGHT_UP_STR => 12,
        VERT_RIGHT_STR => 13,
        HORI_UP_STR => 14,
        CROSS_STR => 15,
        _ => 0b00000000
    }
}













fn add_to_cell<'a>(buffer : &'a mut Buffer, x : u16, y : u16,border : u8,style: &Style) {
    let cell: &mut Cell = buffer.get_mut(x, y);
    if style.fg.is_some() {
        if cell.style().fg.unwrap() == Color::Reset || x%2 == 0{
            cell.set_style(*style);
        }
    }
    cell.set_symbol(add_to(border,&cell.symbol));
}


fn add_to(first: u8, second: &str) -> &'static str {
    return BORDERS[(char_to_border(&second) | first) as usize];
}
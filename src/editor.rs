use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use crate::event::{EventHandler, Event};
use crate::noteblock_widget::{NoteblockWidget};
use crate::parsers::{Song, song, self, Layer, Instrument, NoteblockSection};
use crate::play_song;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, self};
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, error, thread};
use tui::Terminal;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};


/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self) -> AppResult<()> {
        self.terminal.draw(|frame: &mut Frame<'_, B>| render(frame))?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> AppResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}


/// Renders the user interface widgets.
pub fn render<B: Backend>(frame: &mut Frame<'_, B>) {
    let block = NoteblockWidget::default()
    // Borders on every side...
    .borders(Borders::ALL)
    .border_type(BorderType::Plain)
    // The background of the current color...
    .style(Style::default());


    // Render into the first chunk of the layout.
    frame.render_widget(block,frame.size());
}

enum SongEdit {
    Layer(Layer),
    Instrument(Instrument),
    Noteblock(NoteblockSection)
}
struct vec_edit {
    index: u16,
    
}
// pub fn play_updated_song(data_in: Receiver<>){

// }


pub fn loadSong(location : &str) {

    // self.song=Some(song);
    // let (tx, rx) = mpsc::channel();
    // if self.playbackThread.is_some_and(|thread| !thread.is_finished()){
        // self.playbackThread.unwrap().thread().
    // }
    // if self.playback_thread.is_none() {

    //     self.
    // }
    // let val = String::from("hi");
    // tx.send(val).unwrap();

    // let temp = &self.song.unwrap().;
    // let test = Arc::new(self);
    // let handle: thread::JoinHandle<()> = thread::spawn(move || {
        // let val = String::from("hi");
        // tx.send(val).unwrap();
        // play_song(&((&self).song).unwrap());
        // play_song(&(self.clone()).song.unwrap());
    // });
}





/// Handles the tick event of the terminal.
pub fn tick() {}


//this is not a good way to do this but uhhhhhhhhhhh
/* this is blocking */
pub fn start() -> AppResult<()> {
    println!("GO");
    // Create an application.
    // thread::scope(|scope| {

    // Is the application running?
    let mut running = true;
    // song
    let mut song: Arc<Mutex<Option<Song>>> = Arc::new(Mutex::new(None));

    let (
        tx ,
        rx ) = mpsc::channel();




    let mut playback_thread = thread::spawn(move || {
        loop {
            let song: Song = rx.recv().unwrap();
            println!("got a {:?}",song);
            play_song(&song);
        }
    });


    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend).unwrap();
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init().unwrap();

    // Start the main loop.
    while running {
        // Render the user interface.
        tui.draw().unwrap();
        // Handle events.
        match tui.events.next().unwrap() {
            Event::Tick => tick(),
            Event::Key(key_event) => 
                match key_event.code {
                    // Exit application on `ESC` or `q`
                    KeyCode::Esc | KeyCode::Char('q') => {
                        running=false;
                    }
                    // Exit application on `Ctrl-C`
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            running=false;
                        }
                    }
                    // Counter handlers
                    KeyCode::Char('L') => {
                        let location = "8xthing.nbs";
                        let mut f = File::open(format!("songs/{}",location)).unwrap();
                        let mut buffer = vec!();
                        f.read_to_end(&mut buffer).unwrap();
                        let (_, temp) = parsers::song(&buffer).unwrap();
                        // song = Some(temp);
                        // tx.send(song.unwrap().clone()).unwrap();
                    }
                    // KeyCode::Char('T') => {
                    //     tx.send("imposter");
                    // }
                    // Other handlers you could add here.
                    _ => {}
                }
            ,
            Event::Mouse(event) => {
                // println!("moused on {:?}",event)
            }
            Event::Resize(x, y) => {
                // println!("resized to {},{}",x,y)
            }
        }
    }

    // Exit the user interface.
    tui.exit().unwrap();
    // });

    Ok(())
}

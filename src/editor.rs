use crossterm::event::{KeyCode, KeyEvent, MouseEvent, KeyModifiers, self, Event};
use rodio::source::Buffered;
use rodio::{OutputStream, Decoder, Source};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use crate::noteblock_widget::{NoteblockWidget};
use crate::parsers::{Song, song, self, Layer, Instrument, NoteblockSection, Header, Noteblock};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, self};
use std::fs::File;
use std::io::Read;
use std::ops::{AddAssign, Add, Div};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::{io, error, thread};
use ratatui::Terminal;
use ratatui::{
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
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
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
    pub fn draw(&mut self, editor_state: &mut EditorState) -> AppResult<()> {
        if editor_state.song.is_some(){
            let block =
                NoteblockWidget {
                    block_width: 4,
                    block_height: 2
                };
            // if ((editor_state.tick - editor_state.cmp_tick).abs() * block.block_width as f32).floor() < 1_f32 { //if no difference in render, dont
            //     return Ok(());
            // }
            
            // editor_state.cmp_tick = editor_state.tick;
            self.terminal.draw(|frame: &mut Frame<'_, B>| {

                    // Render into the first chunk of the layout.
                    frame.render_stateful_widget(block,frame.size(),editor_state); //we need faster rendering
                
            })?;
        }
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

/*
noteblock, tick (default -1), layer, (default -1), index (-1 if didnt find) */
fn get_next_noteblock(noteblocks: &Vec<NoteblockSection>, start_index: i32) -> (Noteblock,i32,i32,i32) {
    let mut tick = -1;
    let mut layer = -1;
    for index in (start_index as usize)..noteblocks.len() {
        match &noteblocks[index] {
            NoteblockSection::SetTick(num) => {
                tick = *num;
            },
            NoteblockSection::SetLayer(num) => {
                layer = *num;
            },
            NoteblockSection::Noteblock(noteblock) => {
                return (noteblock.clone(),tick,layer,index as i32);
            },
        }
    }
    return (Noteblock {
        instrument: -1,
        key: -1,
        volume: -1,
        panning: u8::MAX,
        pitch: -1,
    },tick,layer,-1);
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum SongEdit {
    Header(Header),
    Layer(Layer,u16),
    Instrument(Instrument,u32),
    Noteblock(NoteblockSection,u16),
    Song(Option<Song>)
}



//todo traverse multiple ticks in a single tick if needed (work at lower gui tick speed)
/// Handles the tick event of the terminal.
fn tick(editor_state: &mut EditorState) {
    if editor_state.playing {
        // println!("|tick|");
        // if(editor_state.index==-1){ //we need to start from beginning then
            
        // }
        let prev_instant = editor_state.prev_instant.elapsed();
        
        if editor_state.prev_tick<editor_state.next_tick {
            editor_state.tick = editor_state.prev_tick as f32+
                ((((editor_state.next_tick-editor_state.prev_tick) as f32)*(prev_instant.as_micros() as f32/editor_state.wait_duration.as_micros() as f32)));

        }
        if prev_instant < editor_state.wait_duration {
            return;
        }
        
        editor_state.prev_index = editor_state.next_index;
        editor_state.prev_tick = editor_state.next_tick;
        editor_state.prev_instant.add_assign(editor_state.wait_duration);

        // let mut tick = get_tick(&editor_state.song.as_mut().unwrap().noteblocks, editor_state.prev_index);
        editor_state.next_index = find_next_tick_index(&editor_state.song.as_mut().unwrap().noteblocks, editor_state.prev_index);
        // let mut new_duration = Duration::from(get_tick(found_index));
        if editor_state.next_index==-1 {
            // println!("found index was {}, from a start index of {}, the tick was {}, and is now {}",found_index,editor_state.index,tick,get_next_loop_tick(tick));
            // tick = get_next_loop_tick(tick);
            let start_tick = editor_state.song.as_ref().unwrap().header.loop_start_tick as i32;
            editor_state.next_index = find_next_index_tick(&editor_state.song.as_mut().unwrap().noteblocks,start_tick);
            // editor_state.next_index = find_next_tick_index(&editor_state.song.as_mut().unwrap().noteblocks, -1);
            // if get_tick(&editor_state.song.as_mut().unwrap().noteblocks,editor_state.next_index)!= editor_state.song.as_mut().unwrap().header.loop_start_tick {
            // }
        }
        
        editor_state.next_tick = get_tick(&editor_state.song.as_mut().unwrap().noteblocks,editor_state.next_index);
        if editor_state.prev_tick>editor_state.next_tick {
            // println!("loop so its ({:?}-{:?})+{:?}",get_next_loop_tick(editor_state.prev_tick),editor_state.prev_tick,editor_state.next_tick);
            editor_state.wait_duration = Duration::from_micros((1000000_f64/(editor_state.tempo)) as u64).mul_f64(((get_next_loop_tick(editor_state.prev_tick)-editor_state.prev_tick)+(editor_state.next_tick-editor_state.song.as_ref().unwrap().header.loop_start_tick as i32)) as f64);
        }else{
            editor_state.wait_duration = Duration::from_micros((1000000_f64/(editor_state.tempo)) as u64).mul_f64((editor_state.next_tick-editor_state.prev_tick) as f64);
        }
        // println!("the tick is {}, last tick is {}",tick,editor_state.last_tick);

    }
    //assume index is the waiting for tick
    
}
fn get_next_loop_tick(tick: i32) -> i32{
    return (((tick+1) as f64/16_f64).ceil()*16_f64) as i32;
}


fn get_tick(noteblocks: &Vec<NoteblockSection>, index: i32) -> i32 {
    if index == -1 {
        return 0;
    }
    match noteblocks[index as usize] {
        NoteblockSection::SetTick(current_tick) => {
            return current_tick;
        },
        NoteblockSection::SetLayer(_) => {},
        NoteblockSection::Noteblock(_) => {},
    }
    todo!();
}

fn find_next_tick_index(noteblocks: &Vec<NoteblockSection>, last_index: i32) -> i32 {
    let mut found_index: i32 = -1;
    // let mut found_tick: i32 = -1;
    for i in ((last_index+1) as usize)..noteblocks.len() {
        match noteblocks[i]{
            NoteblockSection::SetTick(_) => {
                found_index = i as i32;
                // found_tick=tick;
                break;
            },
            NoteblockSection::SetLayer(_) => {},
            NoteblockSection::Noteblock(_) => {},
        }
    }
    // if found_index==-1&&last_index!=-1 {
    //     return find_next_tick_index(noteblocks,-1);
    // }

    return found_index;
}

fn find_next_index_tick(noteblocks: &Vec<NoteblockSection>, tick: i32) -> i32 {
    let mut found_index: i32 = -1;
    // let mut found_tick: i32 = -1;
    for i in 0..noteblocks.len() {
        match noteblocks[i]{
            NoteblockSection::SetTick(num) => {
                if num >= tick {
                    found_index = i as i32;
                    break;
                }
            },
            NoteblockSection::SetLayer(_) => {},
            NoteblockSection::Noteblock(_) => {},
        }
    }
    // if found_index==-1&&last_index!=-1 {
    //     return find_next_tick_index(noteblocks,-1);
    // }

    return found_index;
}

#[derive(Debug)]
pub struct EditorState {
    pub song: Option<Song>,
    pub playing: bool,
    pub tempo: f64,
    pub cmp_tick: f32,
    pub tick: f32,
    pub prev_tick: i32,
    pub next_tick: i32,
    pub prev_index: i32,
    pub next_index: i32,
    pub prev_instant: Instant,
    pub wait_duration: Duration,
    pub debug_instant: Instant,
}



/* this is blocking */
pub fn start() -> AppResult<()> {
    println!("GO");
    // Create an application.
    // thread::scope(|scope| {

    // Is the application running?
    let mut running = true;
    // song

    let (
        tx ,
        rx ) = mpsc::channel();

    

    
    thread::spawn(move || {
        loop {
            let song_edit: SongEdit = rx.recv().unwrap();
            // println!("got a {:?}",song_edit);
            match song_edit {
                SongEdit::Header(_) => todo!(),
                SongEdit::Layer(_, _) => todo!(),
                SongEdit::Instrument(_, _) => todo!(),
                SongEdit::Noteblock(_, _) => todo!(),
                SongEdit::Song(newSong) => {
                    if(newSong.is_some()){
                        start_playing_sound(newSong.unwrap(),&rx);
                    }
                },
            }
            
        }
    });


    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend).unwrap();
    let mut tui = Tui::new(terminal);
    tui.init().unwrap();

    let mut editor_state = EditorState {
        song:None,
        prev_tick:0,
        next_index:0,
        tempo:-1_f64,
        prev_instant:Instant::now(),
        playing: false,
        debug_instant: Instant::now(),
        next_tick: 0,
        prev_index: 0,
        tick: 0_f32,
        cmp_tick: 0_f32,
        wait_duration: Duration::new(0,0),
    };

    let event_wait = Duration::from_secs(0);
    let wait_duration = Duration::from_millis(16);
    let mut last_tick = Instant::now();

    // Start the main loop.
    while running {

        if event::poll(event_wait).expect("no events available") {
            match event::read().expect("unable to read event") {
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
                            let location = "Nyan Cat.nbs";
                            let mut f = File::open(format!("songs/{}",location)).unwrap();
                            let mut buffer = vec!();
                            f.read_to_end(&mut buffer).unwrap();
                            let (_, temp) = parsers::song(&buffer).unwrap();
                            editor_state.tempo = temp.header.tempo as f64 / 100_f64;
                            editor_state.song = Some(temp);
                            editor_state.playing=true;
                            editor_state.prev_instant = Instant::now();
                            editor_state.debug_instant = Instant::now();
                            tx.send(SongEdit::Song(editor_state.song.clone())).unwrap();
                        }
                        // KeyCode::Char('T') => {
                        //     tx.send("imposter");
                        // }
                        // Other handlers you could add here.
                        _ => {}
                    },
                Event::Mouse(e) => {
                    // println!("moused on {:?}",event)
                },
                Event::Resize(w, h) => {
                    // println!("resized to {},{}",x,y)
                },
                _ => unimplemented!(),
            }
        }

        // Render the user interface.
        tui.draw(&mut editor_state).unwrap();
        tick(&mut editor_state);
        std::thread::sleep(wait_duration.saturating_sub(last_tick.elapsed()));
        last_tick=Instant::now();
    }

    // Exit the user interface.
    tui.exit().unwrap();
    // });

    Ok(())
}
const DEFAULT_INSTRUMENTS: [&str; 16] = ["harp","dbass","bdrum","sdrum","click","guitar","flute","bell","icechime","xylobone","iron_xylophone","cow_bell","didgeridoo","bit","banjo","pling"];

fn start_playing_sound(mut song: Song, reciever: &Receiver<SongEdit>){

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sounds: Vec<Buffered<Decoder<File>>> = Vec::new();
    let mut total_instruments: Vec<Instrument> = Vec::new();
    let mut effective_layers: Vec<Layer> = Vec::new();
    // let guard: MutexGuard<'_, Option<Song>> = mutex_song.lock().unwrap();
    // let song_option : Option<Song> = *guard;
    // let binding = mutex_song.lock();
    // let song : &Song = binding.as_ref().unwrap().as_ref().unwrap();
    for name in DEFAULT_INSTRUMENTS {
        // println!("Instrument {:?}",name);
        sounds.push(Decoder::new(std::fs::File::open(format!("sounds/{}.ogg",name)).unwrap()).unwrap().buffered());
        total_instruments.push(Instrument {
            name: name.into(),
            sound_file: name.into(),
            sound_key: 45,
            press_key: 1
        });
    };

    let mut tempo_changer_index: i8 = -1;
    for instrument in song.custom_instruments {
        // println!("Custom instrument {:?} at sounds/{:?}",instrument.name,instrument.sound_file);
        if instrument.name == "Tempo Changer" {
            tempo_changer_index = sounds.len() as i8;
            sounds.push(Decoder::new(std::fs::File::open("sounds/harp.ogg").unwrap()).unwrap().buffered()); //dummy sound because i dont wanna option
            total_instruments.push(instrument);
            continue;
        }
        sounds.push(Decoder::new(std::fs::File::open(format!("sounds/{}",instrument.sound_file)).unwrap()).unwrap().buffered());
        total_instruments.push(instrument);
    }

    if !song.layers.is_empty() {
        for layer in song.layers{
            effective_layers.push(layer);
        }
    } else {
        for i in 0..song.header.layer_count {
            effective_layers.push(Layer {
                name: format!("default_layer_{}",i),
                locked: 0,
                volume: 100,
                stereo: 100
            });
        }
    }
    



    let mut loop_count = 0;
    // println!("tempo is {:?}tps",(song.header.tempo as f64 / 100_f64));


    
    let mut tick_length : f64;
    let mut tick_duration : Duration;
    let mut tick: i32=0;
    let mut last_tick: i32 = 0;
    let unaccuracy: Duration;
    if std::env::consts::OS == "windows" {
        unaccuracy = Duration::new(u64::MAX, 0); // never trust
    }else{
        unaccuracy = Duration::new(0, 5*1000*1000); // 5 ms untrustworthyness
    }
    // drop(song);
    loop {
        tick_length = (100000000_f64/(song.header.tempo as f64)) as f64;
        tick_duration = std::time::Duration::from_micros(tick_length as u64);


        let mut layer_pos: i32=-1;

        let mut mixer: (std::sync::Arc<rodio::dynamic_mixer::DynamicMixerController<f32>>, rodio::dynamic_mixer::DynamicMixer<f32>) = rodio::dynamic_mixer::mixer(2,44100);
        // let mut found = false;
        let mut index: usize=usize::MAX;
        for i in 0..song.noteblocks.len(){
            match &song.noteblocks[i]{
                NoteblockSection::SetTick(check) => 
                    if check>=&tick {
                        last_tick = tick;
                        tick = *check;
                        index=i;
                        break;
                    }
                ,
                NoteblockSection::SetLayer(_) => {},
                NoteblockSection::Noteblock(_) => {},
            }
        }
        if index==usize::MAX{
            println!("Couldn't find starting position for {}",tick);
            break;
        } else {
            // println!("beginning tick {:?} at time {:?}",tick,std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
            // std::thread::sleep(tick_duration.mul_f64((tick-last_tick) as f64));
        }

        let mut lastTime = Instant::now();
        let mut drift: u128 = 0;
        let mut new_tick = -1;
        let start_time = Instant::now();
        loop {
            
            for i in index..song.noteblocks.len() {
                // let section = 
                // println!("doing section {:?}",section);
                index=i;
                match &song.noteblocks[i]{
                    parsers::NoteblockSection::SetTick(num) => {
                        new_tick = *num;
                        break;
                    }
                    parsers::NoteblockSection::SetLayer(num) => layer_pos=i32::from(*num),
                    parsers::NoteblockSection::Noteblock(noteblock) => {
                        if noteblock.instrument == tempo_changer_index {
                            tick_length = (1000000_f64/(noteblock.pitch as f64 / 15_f64)) as f64;
                            tick_duration = std::time::Duration::from_micros(tick_length as u64);
                            // println!("Set tempo to {}",(noteblock.pitch as f64 / 15_f64));
                            continue;
                        }
                        mixer.0.add(
                            sounds[noteblock.instrument as usize].clone()
                            .speed(
                                (2_f64.powf(
                                    (
                                        (
                                            (noteblock.key as f64)-(total_instruments[noteblock.instrument as usize].sound_key as f64)
                                        )+(
                                            (noteblock.pitch as f64)/100_f64
                                        )
                                    )*(1_f64/12_f64))) as f32
                            )
                            
                            .amplify((noteblock.volume as f32 * effective_layers[layer_pos as usize].volume as f32)/10000_f32)
                            .convert_samples()
                        );
                        // println!("noteblock at {:?},{:?}: {:?}", tick,layer_pos, noteblock);
        
                    }
                }
            }
            // print!("waiting from {last_tick} to {tick}, time is {:?} then",start_time.elapsed());
            let duration = tick_duration.mul_f64((tick-last_tick) as f64);
            if !duration.is_zero(){
                if duration > unaccuracy {
                    let mut rcv_iter = reciever.try_iter();
                    let mut rcv = rcv_iter.next();
                    while rcv.is_some() {
                        match rcv.unwrap() {
                            SongEdit::Header(_) => todo!(),
                            SongEdit::Layer(_, _) => todo!(),
                            SongEdit::Instrument(_, _) => todo!(),
                            SongEdit::Noteblock(section, indx) => {
                                song.noteblocks.insert(indx as usize, section);
                                if indx as usize <= index {
                                    index+=1;
                                }
                            },
                            SongEdit::Song(_) => todo!(),
                        }
                        rcv = rcv_iter.next();
                    }
                    std::thread::sleep(duration.saturating_sub(unaccuracy))
                }
                while lastTime.elapsed()<duration { std::hint::spin_loop(); } //accurate waiting
            }
            // println!("{:?}, the duration was {:?}",start_time.elapsed(),duration);
            drift+=lastTime.elapsed().as_nanos()-duration.as_nanos();
            last_tick=tick;
            tick=new_tick;
            // lastTime = Instant::now();
            lastTime.add_assign(duration);
            stream_handle.play_raw(mixer.1.convert_samples()).unwrap();
            
            mixer = rodio::dynamic_mixer::mixer(2,44100);
            index+=1;
            if index==song.noteblocks.len(){
                break;
            }

        }
        // stream_handle.play_raw(mixer.1.convert_samples()).unwrap();
        // mixer = rodio::dynamic_mixer::mixer(2,44100);
        // //play last tick
        // let duration = tick_duration.mul_f64((tick-last_tick) as f64);
        // if duration > unaccuracy {
        //     std::thread::sleep(duration.saturating_sub(unaccuracy))
        // }
        // while lastTime.elapsed()<duration { std::hint::spin_loop(); } //accurate waiting

        // drift+=lastTime.elapsed().as_nanos()-duration.as_nanos();
        // lastTime.add_assign(duration);

        // stream_handle.play_raw(mixer.1.convert_samples()).unwrap();


        //wait til beat
        // println!("time since start1: {:?}",start_time.elapsed());
        // println!("next loop tick is {}, while im at {}",get_next_loop_tick(tick),tick);
        let duration = tick_duration.mul_f64((get_next_loop_tick(tick)-tick) as f64); //loops at the beat
        if duration > unaccuracy {
            std::thread::sleep(duration.saturating_sub(unaccuracy))
        }
        while lastTime.elapsed()<duration { std::hint::spin_loop(); } //accurate waiting
        lastTime.add_assign(duration);

        // println!("Drift was {:?}ms",(drift as f64)/1000000_f64);
        if song.header.looping==0 {break;}
        tick=song.header.loop_start_tick as i32;
        loop_count+=1;
        if loop_count == song.header.loop_count { break; }
        // println!("time since start2: {:?}",start_time.elapsed());

        // println!("====================================================LOOPED====================================================");
    }

    // std::thread::sleep(std::time::Duration::from_millis(1000));
}
use std::io::{self, Read};
use std::thread;
use std::time::Instant;
use std::{fs::File, time::Duration};

use editor::{AppResult};
use parsers::{Song, song};
use rodio::{OutputStream, Decoder, source::Buffered, Source};

use crate::parsers::{Instrument, Layer, NoteblockSection};

mod parsers;
mod editor;
mod noteblock_widget;

const DEFAULT_INSTRUMENTS: [&str; 16] = ["harp","dbass","bdrum","sdrum","click","guitar","flute","bell","icechime","xylobone","iron_xylophone","cow_bell","didgeridoo","bit","banjo","pling"];

fn play_song(song : &Song){

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut last_tick: i32 = -1;
    let mut sounds: Vec<Buffered<Decoder<File>>> = Vec::new();
    let mut total_instruments: Vec<&Instrument> = Vec::new();
    let mut effective_layers: Vec<&Layer> = Vec::new();
    let mut default_instruments: Vec<Instrument> = Vec::new();
    for name in DEFAULT_INSTRUMENTS {
        // println!("Instrument {:?}",name);
        sounds.push(Decoder::new(std::fs::File::open(format!("sounds/{}.ogg",name)).unwrap()).unwrap().buffered());
        default_instruments.push(Instrument {
            name: name.into(),
            sound_file: name.into(),
            sound_key: 45,
            press_key: 1
        });
    };
    for i in 0..default_instruments.len() { //so this is how you rust
        total_instruments.push(&default_instruments[i]);
    };

    let mut tempo_changer_index: i8 = -1;
    for instrument in &song.custom_instruments {
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

    let defaultLayer: Layer = Layer {
        name: "default_layer".to_owned(),
        locked: 0,
        volume: 100,
        stereo: 100
    };
    if !song.layers.is_empty() {
        for layer in &song.layers{
            effective_layers.push(layer);
        }
    } else {
        for _ in 0..song.header.layer_count {
            effective_layers.push(&defaultLayer);
        }
    }
    



    let mut loop_count = 0;
    // println!("tempo is {:?}tps",(song.header.tempo as f64 / 100_f64));


    
    let mut tick_length : f64;
    let mut tick_duration : Duration;
    let mut tick: i32=-1;
    let unaccuracy: Duration;
    if std::env::consts::OS == "windows" {
        unaccuracy = Duration::new(u64::MAX, 0); // never trust
    }else{
        unaccuracy = Duration::new(0, 5*1000*1000); // 5 ms untrustworthyness
    }
    loop {
        tick_length = (100000000_f64/(song.header.tempo as f64)) as f64;
        tick_duration = std::time::Duration::from_micros(tick_length as u64);


        let mut layer_pos: i32=-1;
        let mut iterator = song.noteblocks.iter();
        let mut mixer: (std::sync::Arc<rodio::dynamic_mixer::DynamicMixerController<f32>>, rodio::dynamic_mixer::DynamicMixer<f32>) = rodio::dynamic_mixer::mixer(2,44100);
        
        if iterator.find(|section| {
            match section{
                NoteblockSection::SetTick(check) => {
                    if check>=&tick {
                        last_tick = tick;
                        tick = *check;
                        return true;
                    }else{
                        return false;
                    }
                },
                NoteblockSection::SetLayer(_) => false,
                NoteblockSection::Noteblock(_) => false,
            }
        }).is_none() {
            // println!("Couldn't find starting position for {}",tick);
            break;
        } else {
            // println!("beginning tick {:?} at time {:?}",tick,std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
            // std::thread::sleep(tick_duration.mul_f64((tick-last_tick) as f64));
        }

        let mut lastTime = Instant::now();
        let mut drift: u128 = 0;
        
        for section in iterator {
            // println!("doing section {:?}",section);
            match section{
                parsers::NoteblockSection::SetTick(num) => {

                    let duration = tick_duration.mul_f64((tick-last_tick) as f64);
                    if duration > unaccuracy {
                        std::thread::sleep(duration.saturating_sub(unaccuracy))
                    }
                    while lastTime.elapsed()<duration { std::hint::spin_loop(); } //accurate waiting

                    drift+=lastTime.elapsed().as_nanos()-duration.as_nanos();
                    last_tick=tick;
                    tick=i32::from(*num);
                    lastTime = Instant::now();

                    stream_handle.play_raw(mixer.1.convert_samples()).unwrap();
                    mixer = rodio::dynamic_mixer::mixer(2,44100);
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
        //play last tick
        let duration = tick_duration.mul_f64((tick-last_tick) as f64);
        if duration > unaccuracy {
            std::thread::sleep(duration.saturating_sub(unaccuracy))
        }
        while lastTime.elapsed()<duration { std::hint::spin_loop(); } //accurate waiting

        drift+=lastTime.elapsed().as_nanos()-duration.as_nanos();
        lastTime = Instant::now();

        stream_handle.play_raw(mixer.1.convert_samples()).unwrap();


        //wait til beat
        let duration = tick_duration.mul_f64((tick%4) as f64); //loops at the beat
        if duration > unaccuracy {
            std::thread::sleep(duration.saturating_sub(unaccuracy))
        }
        while lastTime.elapsed()<duration { std::hint::spin_loop(); } //accurate waiting
        

        // println!("Drift was {:?}ms",(drift as f64)/1000000_f64);
        if song.header.looping==0 {break;}
        tick=song.header.loop_start_tick as i32;
        loop_count+=1;
        if loop_count == song.header.loop_count { break; }

        // println!("====================================================LOOPED====================================================");
    }

    std::thread::sleep(std::time::Duration::from_millis(1000));
}


fn main() {
    // let handle = thread::spawn(|| {
        editor::start().unwrap();
        let test: Vec<String> = Vec::new();
    // });
    // let args: Vec<String> = std::env::args().collect();
	// let mut f = File::open("songs/8xthing.nbs").unwrap();
    // if args.len()>1 {
    //     f = File::open(format!("songs/{}",args[1])).unwrap();
    // }
    // let mut buffer = vec!();
    // f.read_to_end(&mut buffer).unwrap();
    // let (_, song) = song(&buffer).unwrap();
    // play_song(song);
    



    

}

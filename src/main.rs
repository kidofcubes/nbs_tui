use std::{fs::File, time::Duration, io::Read};

use parsers::{Song, song};
use rodio::{OutputStream, Decoder, source::Buffered, Source};

mod parsers;

fn play_song(song : Song){

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut lastTick: i32 = -1;
    let mut sounds: Vec<Buffered<Decoder<File>>> = Vec::new();
    for name in ["harp","dbass","bdrum","sdrum","click","guitar","flute","bell","icechime","xylobone","iron_xylophone","cow_bell","didgeridoo","bit","banjo","pling"] {
        println!("Instrument {:?}",name);
        sounds.push(Decoder::new(std::fs::File::open(format!("sounds/{}.ogg",name)).unwrap()).unwrap().buffered());
    };

    for instrument in song.custom_instruments {
        println!("Custom instrument {:?} at {:?}",instrument.name,instrument.sound_file);
        sounds.push(Decoder::new(std::fs::File::open(format!("sounds/{:?}",instrument.sound_file)).unwrap()).unwrap().buffered());
    }


    

    let mut tick: i32=-1;
    let mut layerPos: i32=-1;
    println!("tempo is {:?}",song.header.tempo);

    let mut mixer: (std::sync::Arc<rodio::dynamic_mixer::DynamicMixerController<i16>>, rodio::dynamic_mixer::DynamicMixer<i16>) = rodio::dynamic_mixer::mixer(2,44100);

    
    let tick_length : f64 = (50000000_f64/(song.header.tempo as f64)) as f64;
    let tick_duration : Duration = std::time::Duration::from_micros(tick_length as u64);
    for i in 0..song.noteblocks.len() {
        match &song.noteblocks[i]{
            parsers::NoteblockSection::SetTick(num) => {
                stream_handle.play_raw(mixer.1.convert_samples()).unwrap();
                mixer = rodio::dynamic_mixer::mixer(2,44100);
                tick=i32::from(*num);
                std::thread::sleep(tick_duration.mul_f64((tick-lastTick) as f64));
                println!("tick {:?} at time {:?}",tick,std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
                lastTick=tick;
            }
            parsers::NoteblockSection::SetLayer(num) => layerPos=i32::from(*num),
            parsers::NoteblockSection::Noteblock(noteblock) => {

                mixer.0.add(
                    sounds[noteblock.instrument as usize].clone()
                    .speed((2_f64.powf((((noteblock.key as f64)-45_f64)+((noteblock.pitch as f64)/100_f64))*(1_f64/12_f64))) as f32)
                    .amplify((noteblock.volume as f32)/100_f32)
                    .convert_samples()
                );
                println!("noteblock at {:?},{:?}: {:?}", tick,layerPos, noteblock);

            }
        }
    }
    stream_handle.play_raw(mixer.1.convert_samples()).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
fn main() {
	let mut f = File::open("8xthing.nbs").unwrap();
    let mut buffer = vec!();
    f.read_to_end(&mut buffer).unwrap();
    let (buffer, song) = song(&buffer).unwrap();
    play_song(song);
	// println!("{:?}", song.header);

}

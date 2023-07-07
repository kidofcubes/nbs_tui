use nom::bytes::streaming::take;
use nom::number::streaming::*;
use nom::{
    combinator::map_res,
    IResult,
};
use NoteblockSection::{SetTick, SetLayer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub open_nbs_version: i8,
    pub vanilla_instrument_count: i8,
    pub song_length: i16,
    pub layer_count: i16, // Layer count				The last layer with at least one note block in it, or the last layer that has had its name, volume or stereo changed.
    pub name: String,     // Song name				The name of the song.
    pub author: String,   // Song author				The author of the song.
    pub orig_author: String, // Song original author		The original author of the song.
    pub description: String, // Song description			The description of the song.
    pub tempo: i16, // Song tempo				The tempo of the song multiplied by 100 (for example, 1225 instead of 12.25). Measured in ticks per second.
    pub auto_save: i8, // Auto-saving				Whether auto-saving has been enabled (0 or 1). As of NBS version 4 this value is still saved to the file, but no longer used in the program.
    pub auto_save_period: i8, // Auto-saving duration		The amount of minutes between each auto-save (if it has been enabled) (1-60). As of NBS version 4 this value is still saved to the file, but no longer used in the program.
    pub time_signature: i8, // Time signature			The time signature of the song. If this is 3, then the signature is 3/4. Default is 4. This value ranges from 2-8.
    pub minutes_spent: i32, // Minutes spent			Amount of minutes spent on the project.
    pub left_clicks: i32,   // Left-clicks				Amount of times the user has left-clicked.
    pub right_clicks: i32,  // Right-clicks				Amount of times the user has right-clicked.
    pub noteblocks_added: i32, // Note blocks added		Amount of times the user has added a note block.
    pub noteblocks_removed: i32, // Note blocks removed		The amount of times the user have removed a note block.
    pub original_file_name: String, // MIDI/Schematic file name	If the song has been imported from a .mid or .schematic file, that file name is stored here (only the name of the file, not the path).
    pub looping: i8,                // Loop on/off				Whether looping is on or off. (0 = off, 1 = on)
    pub loop_count: i8, // Max loop count			0 = infinite. Other values mean the amount of times the song loops.
    pub loop_start_tick: i16, // Loop start tick			Determines which part of the song (in ticks) it loops back to.

                              // NOTEBLOCKS
                              // noteblocks: Vec<Noteblock>, // stream when

                              // LAYERS (repeated layer_count times)
                              // layers: Vec<Layer>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Noteblock {
    pub instrument: i8,
    pub key: i8,
    pub volume: i8,
    pub panning: u8,
    pub pitch: i16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layer {
    pub name: String,
    pub locked: i8,
    pub volume: i8,
    pub stereo: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instrument {
    pub name: String,
    pub sound_file: String,
    pub sound_key: i8, //0-87
    pub press_key: i8, //0,1
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub header: Header,
    pub noteblocks: Vec<NoteblockSection>,
    pub layers: Vec<Layer>,
    pub custom_instruments: Vec<Instrument>,
}

pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, _is_open_nbs) = le_i16(input)?;
	//shigsplode when is_open_nbs is not 0
    let (input, open_nbs_version) = le_i8(input)?;
    let (input, vanilla_instrument_count) = le_i8(input)?;
    let (input, song_length) = le_i16(input)?;
    let (input, layer_count) = le_i16(input)?;
    let (input, string_length) = le_i32(input)?;
    let (input, name) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, string_length) = le_i32(input)?;
    let (input, author) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, string_length) = le_i32(input)?;
    let (input, orig_author) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, string_length) = le_i32(input)?;
    let (input, description) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, tempo) = le_i16(input)?;
    let (input, auto_save) = le_i8(input)?;
    let (input, auto_save_period) = le_i8(input)?;
    let (input, time_signature) = le_i8(input)?;
    let (input, minutes_spent) = le_i32(input)?;
    let (input, left_clicks) = le_i32(input)?;
    let (input, right_clicks) = le_i32(input)?;
    let (input, noteblocks_added) = le_i32(input)?;
    let (input, noteblocks_removed) = le_i32(input)?;
    let (input, string_length) = le_i32(input)?;
    let (input, original_file_name) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, looping) = le_i8(input)?;
    let (input, loop_count) = le_i8(input)?;
    let (input, loop_start_tick) = le_i16(input)?;

    return Ok((
        input,
        Header {
            vanilla_instrument_count,
            song_length,
            layer_count,
            open_nbs_version,
            name: name.into(),
            author: author.into(),
            orig_author: orig_author.into(),
            description: description.into(),
            tempo,
            auto_save,
            auto_save_period,
            time_signature,
            minutes_spent,
            left_clicks,
            right_clicks,
            noteblocks_added,
            noteblocks_removed,
            original_file_name: original_file_name.into(),
            looping,
            loop_count,
            loop_start_tick,
        },
    ));
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoteblockSection {
    SetTick(i32),
    SetLayer(i32),
    Noteblock(Noteblock),
}

pub fn noteblock(input: &[u8]) -> IResult<&[u8], Noteblock> {
    let (input, instrument) = le_i8(input)?;
    let (input, key) = le_i8(input)?;
    let (input, volume) = le_i8(input)?;
    let (input, panning) = le_u8(input)?;
    let (input, pitch) = le_i16(input)?;
    return Ok((input,Noteblock {
        instrument,
        key,
        volume,
        panning,
        pitch
    }))
}
pub fn layer(input: &[u8]) -> IResult<&[u8], Layer> {
    let (input, string_length) = le_i32(input)?;
    let (input, name) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, locked) = le_i8(input)?; //1 is locked
    let (input, volume) = le_i8(input)?; //0-100
    let (input, stereo) = le_u8(input)?;
    return Ok((input, Layer {
        name: name.into(),
        locked,
        volume,
        stereo,

    }))
}
pub fn custom_instrument(input: &[u8]) -> IResult<&[u8], Instrument> {
    let (input, string_length) = le_i32(input)?;
    let (input, name) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, string_length) = le_i32(input)?;
    let (input, sound_file) = map_res(take(string_length as usize), std::str::from_utf8)(input)?;
    let (input, sound_key) = le_i8(input)?;
    let (input, press_key) = le_i8(input)?;
    return Ok((input, Instrument {
        name: name.into(),
        sound_file: sound_file.into(),
        sound_key,
        press_key,
    }))
}

pub fn song(input: &[u8]) -> IResult<&[u8], Song> {
    let (mut input, header) = header(input)?;
    //NOTEBLOCKS
    let mut noteblocks: Vec<NoteblockSection> = Vec::new();
    let mut tick: i32=-1;
    let mut layer_pos: i32=-1;
    loop {
        if layer_pos==-1 {
            let tick_jump = le_i16(input)?; input = tick_jump.0;
            if tick_jump.1!=0 {
                // println!("tick_jump was {:?}",tick_jump.1);
                tick+=i32::from(tick_jump.1);
                noteblocks.push(SetTick(tick));
                // continue;
            }else{
                break;
            }
        }
        let layer_jump = le_i16(input)?; input = layer_jump.0;

        if layer_jump.1==0 {
            layer_pos = -1;
            continue;
        }
        layer_pos+=i32::from(layer_jump.1);
        noteblocks.push(SetLayer(layer_pos));

        let out: (&[u8], Noteblock) = noteblock(&input).unwrap();
        input=out.0;
        noteblocks.push(NoteblockSection::Noteblock(out.1));
    }
    let mut layers: Vec<Layer> = Vec::new();
    let mut custom_instruments: Vec<Instrument> = Vec::new();

    //LAYERS (optional)
    if !input.is_empty() {
        for _ in 0..header.layer_count {
            let layer = layer(input)?; input = layer.0;
            layers.push(layer.1);
        }
    
        //CUSTOM INSTRUMENTS (optional)
        if !input.is_empty() {
            let custom_instruments_length = le_u8(input)?; input = custom_instruments_length.0;
            for _ in 0..custom_instruments_length.1 {
                let custom_instrument = custom_instrument(input)?; input = custom_instrument.0;
                custom_instruments.push(custom_instrument.1);
            }
        }
    }
    return Ok((input, Song {
        header,
        noteblocks,
        layers,
        custom_instruments,
    }))


}
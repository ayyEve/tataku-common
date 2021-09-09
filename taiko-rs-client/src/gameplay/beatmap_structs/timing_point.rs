///https://osu.ppy.sh/wiki/en/osu%21_File_Formats/Osu_%28file_format%29#timing-points
#[derive(Clone, Copy)]
pub struct TimingPoint {
    /// Start time of the timing section, in milliseconds from the beginning of the beatmap's audio. The end of the timing section is the next timing point's time (or never, if this is the last timing point).
    pub time: f32,
    /// This property has two meanings:
    ///     For uninherited timing points, the duration of a beat, in milliseconds.
    ///     For inherited timing points, a negative inverse slider velocity multiplier, as a percentage. For example, -50 would make all sliders in this timing section twice as fast as SliderMultiplier.
    pub beat_length: f32,
    /// Volume percentage for hit objects
    pub volume: u32,
    /// Amount of beats in a measure. Inherited timing points ignore this property.
    pub meter: u32,

    // effects

    /// Whether or not kiai time is enabled
    pub kiai: bool,
    /// Whether or not the first barline is omitted in osu!taiko and osu!mania
    pub skip_first_barline: bool,

    // samples

    /// Default sample set for hit objects (0 = beatmap default, 1 = normal, 2 = soft, 3 = drum)
    pub sample_set: u32,
    /// Custom sample index for hit objects. 0 indicates osu!'s default hitsounds
    pub sample_index: u32
}
impl TimingPoint {
    pub fn from_str(str:&str) -> TimingPoint {
        // time,beatLength,meter,sampleSet,sampleIndex,volume,uninherited,effects
        // println!("{}", str.clone());
        let mut split = str.split(',');
        let time = split.next().unwrap_or("0").parse::<f32>().unwrap_or(0.0);
        let beat_length = split.next().unwrap_or("0").parse::<f32>().unwrap_or(0.0);
        let meter = split.next().unwrap_or("4").parse::<u32>().unwrap_or(4);
        let sample_set = split.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        let sample_index = split.next().unwrap_or("0").parse::<u32>().unwrap_or(0);

        let volume = match split.next() {
            Some(str) => str.parse::<u32>().unwrap_or(50),
            None => 50
        };
        let _uninherited = split.next();
        let effects = match split.next() {
            Some(str) => str.parse::<u32>().unwrap_or(0),
            None => 0
        };

        let kiai = (effects & 1) == 1;
        let skip_first_barline = (effects & 8) == 1;

        TimingPoint {
            time, 
            beat_length, 
            volume, 
            meter,

            sample_set,
            sample_index,

            kiai,
            skip_first_barline
        }
    }

    pub fn is_inherited(&self) -> bool {
        return self.beat_length < 0.0;
    }


    pub fn bpm_multiplier(&self) -> f32 {
        if self.beat_length > 0.0 {1.0}
        else {(-self.beat_length as f32).clamp(10.0, 1000.0) / 100.0}
    }
}



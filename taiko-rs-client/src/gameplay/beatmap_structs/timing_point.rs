//TODO! implement time signature, its used for barlines

#[derive(Clone)]
pub struct TimingPoint {
    /// Start time of the timing section, in milliseconds from the beginning of the beatmap's audio. The end of the timing section is the next timing point's time (or never, if this is the last timing point).
    pub time: f64,
    /// This property has two meanings:
    ///     For uninherited timing points, the duration of a beat, in milliseconds.
    ///     For inherited timing points, a negative inverse slider velocity multiplier, as a percentage. For example, -50 would make all sliders in this timing section twice as fast as SliderMultiplier.
    pub beat_length: f32,
    /// Volume percentage for hit objects
    pub volume: u32,
    pub kiai: bool
}
impl TimingPoint {
    pub fn from_str(str:&str) -> TimingPoint {
        // time,beatLength,meter,sampleSet,sampleIndex,volume,uninherited,effects
        // println!("{}", str.clone());
        let mut split = str.split(',');
        let time = split.next().unwrap().parse::<f64>().unwrap().round();
        let beat_length = split.next().unwrap().parse::<f32>().unwrap();
        let _meter = split.next(); //.unwrap().parse::<u32>().unwrap();
        let _sample_set = split.next(); //.unwrap().parse::<u32>().unwrap();
        let _sample_index = split.next(); //.unwrap().parse::<u32>().unwrap();
        let volume = split.next().unwrap().parse::<u32>().unwrap_or(50);
        let _uninherited = split.next();
        let effects = split.next().unwrap().parse::<u32>().unwrap();

        let kiai = (effects & 1) == 1;

        TimingPoint {
            time, 
            beat_length, 
            volume, 
            kiai
        }
    }

    pub fn is_inherited(&self) -> bool {
        return self.beat_length < 0.0;
    }
}
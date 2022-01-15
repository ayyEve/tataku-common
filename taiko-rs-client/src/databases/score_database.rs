use taiko_rs_common::{serialization::*, types::{PlayMode, Replay, Score}};
use crate::REPLAYS_DIR;

pub fn get_scores(hash:&String, playmode:PlayMode) -> Vec<Score> {
    let db = crate::databases::DATABASE.lock();
    let mut s = db.prepare(&format!("SELECT * FROM scores WHERE map_hash='{}' AND playmode={}", hash, playmode as u8)).unwrap();

    s.query_map([], |r| {
        let _score_hash:String = r.get("score_hash")?;

        let score = Score {
            version: r.get("version").unwrap_or(1), // v1 didnt include version in the table
            username: r.get("username")?,
            playmode: r.get::<&str, u8>("playmode")?.into(),
            score: r.get("score")?,
            combo: r.get("combo")?,
            max_combo: r.get("max_combo")?,
            x50: r.get("x50").unwrap_or(0),
            x100: r.get("x100")?,
            x300: r.get("x300")?,
            xmiss: r.get("xmiss")?,
            xgeki: r.get("xgeki").unwrap_or_default(),
            xkatu: r.get("xkatu").unwrap_or_default(),
            accuracy: r.get("accuracy").unwrap_or_default(),
            beatmap_hash: r.get("map_hash")?,
            speed: r.get("speed").unwrap_or(1.0),
            hit_timings: Vec::new(),
            replay_string: None
        };

        Ok(score)
    })
        .unwrap()
        // .filter_map(|m|m.ok())
        .filter_map(|m| {
            if let Err(e) = &m {
                println!("score error: {}", e);
            }
            m.ok()
        })
        .collect::<Vec<Score>>()
}

pub fn save_score(s:&Score) {
    println!("saving score");
    let db = crate::databases::DATABASE.lock();
    let sql = format!(
        "INSERT INTO scores (
            map_hash, score_hash,
            username, playmode,
            score,
            combo, max_combo,
            x50, x100, x300, geki, katu, xmiss,
            speed, 
            version
        ) VALUES (
            '{}', '{}',
            '{}', {},
            {},
            {}, {},
            {}, {}, {}, {}, {}, {},
            {},
            {}
        )", 
        s.beatmap_hash, s.hash(),
        s.username, s.playmode as u8,
        s.score,
        s.combo, s.max_combo,
        s.x50, s.x100, s.x300, s.xgeki, s.xkatu, s.xmiss, 
        s.speed,
        s.version
    );
    let mut s = db.prepare(&sql).unwrap();
    s.execute([]).unwrap();
}


pub fn save_replay(r:&Replay, s:&Score) -> std::io::Result<()> {
    let mut writer = SerializationWriter::new();
    writer.write(r.clone());

    let filename = format!("{}/{}.rs_replay", REPLAYS_DIR,s.hash());
    save_database(&filename, writer)
}

pub fn get_local_replay(score_hash:String) -> std::io::Result<Replay> {
    let fullpath = format!("{}/{}.rs_replay", REPLAYS_DIR, score_hash);
    println!("[Replay] loading replay: {}", fullpath);
    let mut reader = open_database(&fullpath)?;
    Ok(reader.read())
}

use taiko_rs_common::{serialization::*, types::{Replay, Score}};
use crate::REPLAYS_DIR;

pub fn get_scores(hash:&String) -> Vec<Score> {
    let db = crate::databases::DATABASE.lock();
    let mut s = db.prepare(&format!("SELECT * FROM scores WHERE map_hash='{}'", hash)).unwrap();

    s.query_map([], |r| {
        let _score_hash:String = r.get("score_hash")?;

        let score = Score {
            username: r.get("username")?,
            playmode: r.get::<&str, u8>("playmode")?.into(),
            score: r.get("score")?,
            combo: r.get("combo")?,
            max_combo: r.get("max_combo")?,
            x50: r.get("x50").unwrap_or(0),
            x100: r.get("x100")?,
            x300: r.get("x300")?,
            geki: r.get("geki")?,
            katu: r.get("katu").unwrap_or(0),
            xmiss: r.get("xmiss").unwrap_or(0),
            beatmap_hash: r.get("map_hash")?,
            hit_timings: Vec::new()
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
            x50, x100, x300, geki, katu, xmiss
        ) VALUES (
            '{}', '{}',
            '{}', {},
            {},
            {}, {},
            {}, {}, {}, {}, {}, {}
        )", 
        s.beatmap_hash, s.hash(),
        s.username, s.playmode as u8,
        s.score,
        s.combo, s.max_combo,
        s.x50, s.x100, s.x300, s.geki, s.katu, s.xmiss
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
    let mut reader = open_database(&fullpath)?;
    Ok(reader.read())
}
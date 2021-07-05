use rocket::{Data, data::ToByteUnit};
use taiko_rs_common::{serialization::SerializationReader, types::Score};

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/score_submit", data = "<data>")]
async fn score_submit(data:Data<'_>) -> std::io::Result<()> {
    let mut bytes:Vec<u8> = Vec::new();

    match data.open(1.gigabytes()).into_bytes().await {
        Ok(capped_bytes) => {
            capped_bytes.iter().for_each(|b|bytes.push(*b));
        }
        Err(e) => {
            println!("error reading score: {}", e);
            return Err(e);
        }
    }

    let mut reader = SerializationReader::new(bytes);
    let score: Score = reader.read();
    println!("got score: {:?}", score);

    Ok(())
}


#[launch]
pub fn rocket() -> _ {
    rocket::build().mount("/", routes![index, score_submit])
}

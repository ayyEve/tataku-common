
use crate::prelude::*;
use sea_orm::{DbBackend, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, Statement, FromQueryResult};


static DATABASE:OnceCell<DatabaseConnection> = OnceCell::const_new();

pub struct Database;

// init
impl Database {
    pub async fn init(settings: &Settings) {
        let conn_str = settings.postgres.connection_string();
        let db = sea_orm::Database::connect(conn_str).await.expect("Error connecting to database");

        println!("[Startup] db connected");
        DATABASE.force_set(db);
    }
}

// database getters
impl Database {
    pub async fn get_user_score_info(user_id: u32, mode: PlayMode) -> (i64, i64, f64, i32, i32) {
        let mut ranked_score = 0;
        let mut total_score = 0;
        let mut accuracy = 0.0;
        let mut playcount = 0;
        let mut rank = 0;
    
        match user_data_table::Entity::find()
            .filter(user_data_table::Column::Mode.eq(mode as i16))
            .filter(user_data_table::Column::UserId.eq(user_id))
            .one(DATABASE.get().unwrap())
            .await {
            Ok(user_data) => {
                match user_data {
                    Some(user_data) => {
                        ranked_score = user_data.ranked_score;
                        total_score = user_data.total_score;
                        accuracy = user_data.accuracy;
                        playcount = user_data.play_count;
                    }
                    None => { }
                };
            },
            Err(e) => println!("[Database] Error: {}", e)
        }
    
        #[derive(Debug, FromQueryResult)]
        struct RankThing {rank: i64}
    
        let things: Vec<RankThing> = RankThing::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT rank FROM (SELECT user_id, ROW_NUMBER() OVER(ORDER BY ranked_score DESC) AS rank FROM user_data WHERE mode=$1) t WHERE user_id=$2"#,
            vec![(mode as i32).into(), (user_id as i32).into()],
        ))
            .all(DATABASE.get().unwrap())
            .await
            .unwrap();
    
        if let Some(thing) = things.first() {
            rank = thing.rank
        }
    
        (ranked_score, total_score, accuracy, playcount, rank as i32)
    }
    
    pub async fn get_user_by_username(username: &String) -> Option<users_table::Model> {
        users_table::Entity::find()
            .filter(users_table::Column::Username.eq(username.clone()))
            .one(DATABASE.force_get())
            .await
            .unwrap()
    }


    pub async fn insert_into_message_history(user_id: u32, channel: String, contents: String) {
        let message_history_entry: message_history_table::ActiveModel = message_history_table::ActiveModel {
            user_id: Set(user_id as i64),
            channel: Set(channel),
            contents: Set(contents),
            ..Default::default()
        };

        if let Err(e) = message_history_entry.insert(DATABASE.force_get()).await {
            println!("[Database] Error inserting into message_history: {}", e);
        }
    }

}
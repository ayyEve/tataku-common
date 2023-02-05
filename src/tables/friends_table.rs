use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_friends")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i32,
    #[sea_orm(primary_key)]
    pub friend_id: i32
}

impl Model {
    pub async fn get_user_friends<'a>(user_id: u32, db: &'a DatabaseConnection) -> Vec<u32> {
        Entity::find()
        .filter(Column::UserId.eq(user_id as i32))
        .all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|a|a.friend_id as u32)
        .collect()
    }

    pub async fn remove_friend<'a>(user_id: u32, friend_id: u32, db: &'a DatabaseConnection) {
        Entity::delete_by_id((user_id as i32, friend_id as i32))
        .exec(db)
        .await
        .unwrap();
    }

    pub async fn add_friend<'a>(user_id: u32, friend_id: u32, db: &'a DatabaseConnection) {
        let mut on_conflict = sea_orm::sea_query::OnConflict::new();
        on_conflict.do_nothing();

        Entity::insert(ActiveModel {
            user_id: sea_orm::Set(user_id as i32),
            friend_id: sea_orm::Set(friend_id as i32),
        })
        .on_conflict(on_conflict)
        .exec(db)
        .await
        .unwrap();
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }

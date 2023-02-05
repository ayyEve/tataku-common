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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation { }

impl ActiveModelBehavior for ActiveModel { }

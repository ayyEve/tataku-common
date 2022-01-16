

pub mod users_table;
pub use users_table::*;


pub mod user_data_table;
pub use user_data_table::*;


pub mod message_history_table;
pub use message_history_table::*;



// scores table
pub mod scores_table;
pub use scores_table::*;
pub use scores_table::Entity as Scores;
pub use scores_table::Model as ScoresModel;
pub use scores_table::ActiveModel as ScoresActiveModel;
pub use sea_orm_migration::prelude::*;

mod m20260520_150438_init_user;
mod m20260521_140248_init_friendship;
mod m20260523_090256_init_session;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260520_150438_init_user::Migration),
            Box::new(m20260521_140248_init_friendship::Migration),
            Box::new(m20260523_090256_init_session::Migration),
        ]
    }
}

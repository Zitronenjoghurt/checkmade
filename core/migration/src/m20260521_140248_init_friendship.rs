use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Friendship::Table)
                    .if_not_exists()
                    .col(uuid(Friendship::RequesterId))
                    .col(uuid(Friendship::AddresseeId))
                    .col(tiny_integer(Friendship::Status).default(Expr::val(0)))
                    .col(timestamp(Friendship::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Friendship::UpdatedAt).default(Expr::current_timestamp()))
                    .primary_key(
                        Index::create()
                            .col(Friendship::RequesterId)
                            .col(Friendship::AddresseeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friendship::Table, Friendship::RequesterId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friendship::Table, Friendship::AddresseeId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "-- Prevent A->B and B->A from coexisting
                 CREATE UNIQUE INDEX idx_friendship_pair
                    ON friendship (LEAST(requester_id, addressee_id), GREATEST(requester_id, addressee_id));

                 -- No self-friending
                 ALTER TABLE friendship
                     ADD CONSTRAINT chk_no_self_friend
                     CHECK (requester_id <> addressee_id);

                 CREATE TRIGGER trigger_friendship_updated_at
                     BEFORE UPDATE ON friendship
                     FOR EACH ROW
                     EXECUTE FUNCTION set_updated_at();",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS trigger_friendship_updated_at ON friendship;
                 DROP INDEX IF EXISTS idx_friendship_pair;
                 ALTER TABLE friendship DROP CONSTRAINT IF EXISTS chk_no_self_friend;",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Friendship::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Friendship {
    Table,
    RequesterId,
    AddresseeId,
    Status,
    CreatedAt,
    UpdatedAt,
}

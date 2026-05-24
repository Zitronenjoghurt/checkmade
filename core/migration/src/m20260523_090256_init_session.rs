use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(pk_uuid(Session::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(small_integer(Session::Status).default(0))
                    .col(boolean(Session::Public).default(false))
                    .col(uuid(Session::WhiteId))
                    .col(uuid(Session::BlackId))
                    .col(blob(Session::Data))
                    .col(timestamp(Session::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Session::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Session::Table, Session::WhiteId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Session::Table, Session::BlackId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER trigger_session_updated_at
                    BEFORE UPDATE ON \"session\"
                    FOR EACH ROW
                    EXECUTE FUNCTION set_updated_at();",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SessionRequest::Table)
                    .if_not_exists()
                    .col(pk_uuid(SessionRequest::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(uuid(SessionRequest::RequesterId))
                    .col(uuid_null(SessionRequest::AddresseeId))
                    .col(blob(SessionRequest::Config))
                    .col(boolean(SessionRequest::Public).default(false))
                    .col(timestamp(SessionRequest::CreatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionRequest::Table, SessionRequest::RequesterId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionRequest::Table, SessionRequest::AddresseeId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SessionRequest::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS trigger_session_updated_at ON \"session\";")
            .await?;

        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
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
enum Session {
    Table,
    Id,
    Status,
    Public,
    WhiteId,
    BlackId,
    Data,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SessionRequest {
    Table,
    Id,
    RequesterId,
    AddresseeId,
    Config,
    Public,
    CreatedAt,
}

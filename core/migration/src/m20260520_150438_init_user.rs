use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_uuid(User::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string(User::Username).unique_key())
                    .col(string(User::FriendCode).unique_key())
                    .col(big_integer(User::Permissions).default(Expr::val(0)))
                    .col(small_integer(User::RateLimitInfractions).default(Expr::val(0)))
                    .col(timestamp(User::LastLogin).default(Expr::current_timestamp()))
                    .col(timestamp(User::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(User::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION set_updated_at()
                     RETURNS TRIGGER AS $$
                     BEGIN
                         NEW.updated_at = now();
                         RETURN NEW;
                     END;
                     $$ LANGUAGE plpgsql;

                CREATE TRIGGER trigger_user_updated_at
                    BEFORE UPDATE ON \"user\"
                    FOR EACH ROW
                    EXECUTE FUNCTION set_updated_at();",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserIdentity::Table)
                    .if_not_exists()
                    .col(uuid(UserIdentity::UserId))
                    .col(small_integer(UserIdentity::Provider))
                    .col(string(UserIdentity::Identifier))
                    .col(timestamp(UserIdentity::CreatedAt).default(Expr::current_timestamp()))
                    .primary_key(
                        Index::create()
                            .col(UserIdentity::UserId)
                            .col(UserIdentity::Provider)
                            .col(UserIdentity::Identifier),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserIdentity::Table, UserIdentity::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_identity_provider_identifier")
                    .table(UserIdentity::Table)
                    .col(UserIdentity::Provider)
                    .col(UserIdentity::Identifier)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_identity_provider_identifier")
                    .table(UserIdentity::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserIdentity::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS trigger_user_updated_at ON \"user\";
                     DROP FUNCTION IF EXISTS set_updated_at;",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Username,
    FriendCode,
    Permissions,
    RateLimitInfractions,
    LastLogin,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserIdentity {
    Table,
    UserId,
    Provider,
    Identifier,
    CreatedAt,
}

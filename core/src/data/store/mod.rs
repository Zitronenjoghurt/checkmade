use crate::error::{CoreError, CoreResult};
use futures::stream::BoxStream;
use futures::StreamExt;
use sea_orm::sea_query::IntoCondition;
use sea_orm::*;

pub mod friendship;
pub mod user;

#[async_trait::async_trait]
pub trait Store {
    type Entity: EntityTrait<Model: Sync>;
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity> + ActiveModelBehavior + Send;

    fn db(&self) -> &DatabaseConnection;

    fn paginate(&self) -> Paginate<'_, Self::Entity> {
        Paginate::new(self.db())
    }

    async fn find_by_id<K>(&self, id: K) -> CoreResult<Option<<Self::Entity as EntityTrait>::Model>>
    where
        K: Into<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
    {
        Self::Entity::find_by_id(id)
            .one(self.db())
            .await
            .map_err(Into::into)
    }

    async fn find_one_by<F>(
        &self,
        filter: F,
    ) -> CoreResult<Option<<Self::Entity as EntityTrait>::Model>>
    where
        F: IntoCondition + Send,
    {
        Self::Entity::find()
            .filter(filter)
            .one(self.db())
            .await
            .map_err(Into::into)
    }

    async fn get_all_stream<'a>(
        &'a self,
    ) -> CoreResult<BoxStream<'a, CoreResult<<Self::Entity as EntityTrait>::Model>>> {
        let stream = Self::Entity::find()
            .stream(self.db())
            .await
            .map_err(Into::<CoreError>::into)?;
        Ok(stream.map(|res| res.map_err(Into::into)).boxed())
    }

    async fn stream_by<'a, F>(
        &'a self,
        filter: F,
    ) -> CoreResult<BoxStream<'a, CoreResult<<Self::Entity as EntityTrait>::Model>>>
    where
        F: IntoCondition + Send,
    {
        let stream = Self::Entity::find()
            .filter(filter)
            .stream(self.db())
            .await
            .map_err(Into::<CoreError>::into)?;
        Ok(stream.map(|res| res.map_err(Into::into)).boxed())
    }

    async fn insert(
        &self,
        model: Self::ActiveModel,
    ) -> CoreResult<<Self::Entity as EntityTrait>::Model>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self::ActiveModel>,
    {
        model.insert(self.db()).await.map_err(Into::into)
    }

    async fn update(
        &self,
        model: Self::ActiveModel,
    ) -> CoreResult<<Self::Entity as EntityTrait>::Model>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self::ActiveModel>,
    {
        model.update(self.db()).await.map_err(Into::into)
    }

    async fn delete_by_id<K>(&self, id: K) -> CoreResult<DeleteResult>
    where
        K: Into<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> + Send,
    {
        Self::Entity::delete_by_id(id)
            .exec(self.db())
            .await
            .map_err(Into::into)
    }

    async fn create_if_not_exists<K, F>(
        &self,
        id: K,
        create: F,
    ) -> CoreResult<<Self::Entity as EntityTrait>::Model>
    where
        K: Into<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
            + Send
            + Clone,
        F: FnOnce() -> Self::ActiveModel + Send,
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self::ActiveModel>,
    {
        let existing = Self::Entity::find_by_id(id.clone())
            .one(self.db())
            .await
            .map_err(CoreError::from)?;

        match existing {
            Some(model) => Ok(model),
            None => create().insert(self.db()).await.map_err(CoreError::from),
        }
    }

    async fn count<F>(&self, filter: F) -> CoreResult<u64>
    where
        F: IntoCondition + Send,
    {
        PaginatorTrait::count(Self::Entity::find().filter(filter), self.db())
            .await
            .map_err(Into::into)
    }

    async fn count_all(&self) -> CoreResult<u64> {
        PaginatorTrait::count(Self::Entity::find(), self.db())
            .await
            .map_err(Into::into)
    }
}

pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total_items: u64,
    pub total_pages: u64,
}

impl<T> Page<T> {
    pub fn has_next(&self) -> bool {
        self.page + 1 < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 0
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            items: self.items.into_iter().map(f).collect(),
            page: self.page,
            page_size: self.page_size,
            total_items: self.total_items,
            total_pages: self.total_pages,
        }
    }

    pub fn try_map<U, E>(self, f: impl FnMut(T) -> Result<U, E>) -> Result<Page<U>, E> {
        Ok(Page {
            items: self
                .items
                .into_iter()
                .map(f)
                .collect::<Result<Vec<_>, _>>()?,
            page: self.page,
            page_size: self.page_size,
            total_items: self.total_items,
            total_pages: self.total_pages,
        })
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Page<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

pub struct Paginate<'a, E: EntityTrait> {
    db: &'a DatabaseConnection,
    select: Select<E>,
    page_size: u64,
}

impl<'a, E> Paginate<'a, E>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
{
    pub(crate) fn new(db: &'a DatabaseConnection) -> Self {
        Self {
            db,
            select: E::find(),
            page_size: 20,
        }
    }

    pub fn filter<F: IntoCondition>(mut self, filter: F) -> Self {
        self.select = self.select.filter(filter);
        self
    }

    pub fn order_by<C: ColumnTrait>(mut self, col: C, order: Order) -> Self {
        self.select = self.select.order_by(col, order);
        self
    }

    pub fn order_by_asc<C: ColumnTrait>(self, col: C) -> Self {
        self.order_by(col, Order::Asc)
    }

    pub fn order_by_desc<C: ColumnTrait>(self, col: C) -> Self {
        self.order_by(col, Order::Desc)
    }

    pub fn page_size(mut self, size: u64) -> Self {
        self.page_size = if size < 1 { 1 } else { size };
        self
    }

    pub async fn fetch_page(&self, page: u64) -> CoreResult<Page<E::Model>> {
        let paginator = self.select.clone().paginate(self.db, self.page_size);

        let total_items = paginator.num_items().await.map_err(CoreError::from)?;
        let size = if self.page_size == 0 {
            1
        } else {
            self.page_size
        };
        let total_pages = total_items.div_ceil(size);
        let items = paginator.fetch_page(page).await.map_err(CoreError::from)?;

        Ok(Page {
            items,
            page,
            page_size: self.page_size,
            total_items,
            total_pages,
        })
    }

    pub async fn count(&self) -> CoreResult<u64> {
        PaginatorTrait::count(self.select.clone(), self.db)
            .await
            .map_err(Into::into)
    }

    pub fn into_page_stream(self) -> BoxStream<'a, CoreResult<Vec<E::Model>>> {
        let db = self.db;
        let page_size = self.page_size;
        let select = self.select;
        futures::stream::unfold(0u64, move |offset| {
            let query = select.clone();
            async move {
                let query = query.offset(Some(offset)).limit(Some(page_size));
                match query.all(db).await {
                    Ok(items) if items.is_empty() => None,
                    Ok(items) => Some((Ok(items), offset + page_size)),
                    Err(e) => Some((Err(e.into()), u64::MAX)),
                }
            }
        })
        .boxed()
    }

    pub fn into_item_stream(self) -> BoxStream<'a, CoreResult<E::Model>> {
        self.into_page_stream()
            .flat_map(|result| match result {
                Ok(items) => futures::stream::iter(items.into_iter().map(Ok)).boxed(),
                Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
            })
            .boxed()
    }
}

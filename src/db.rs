use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{
    sql::{Id, Thing},
    Connection, Surreal,
};

#[async_trait]
pub trait DbItem<C: Connection>: Serialize + DeserializeOwned {
    const NAME: &'static str;
    type GetQuery;

    fn get_id(&self) -> Id;

    fn as_thing(&self) -> Thing {
        Thing {
            id: self.get_id().clone(),
            tb: Self::NAME.to_string(),
        }
    }

    async fn get_from_db(
        db: &Surreal<C>,
        query: Self::GetQuery,
    ) -> Result<Option<Self>, surrealdb::Error>;

    async fn update_in_db(&self, db: &Surreal<C>) -> Result<Option<Self>, surrealdb::Error> {
        db.update(self.as_thing()).content(self).await
    }

    async fn delete_from_db(&self, db: &Surreal<C>) -> Result<Option<Self>, surrealdb::Error> {
        db.delete(Thing {
            id: self.get_id().clone(),
            tb: Self::NAME.to_string(),
        })
        .await
    }
}

const DB_PATH: &str = "weathurber.db";

pub(crate) struct Database {
    db: surrealdb::Datastore,
}

impl Database {
    fn new() -> Self {
        let db = async {
            surrealdb::Datastore::new(format!("file://{DB_PATH}").as_str())
                .await
                .expect("unable to create datastore")
        };

        Self { db }
    }

    async fn set(&mut self, key: surrealdb::Key, value: surrealdb::Val) {
        let ds = &self.db;
        let mut transaction = ds
            .transaction(true, true)
            .await
            .expect("unable to start transaction");
        transaction.set(key, value).await.expect("unable to set");
        transaction.commit();
    }

    async fn get(&mut self, key: surrealdb::Key) -> surrealdb::Val {
        let ds = &self.db;
        let mut transaction = ds
            .transaction(false, true)
            .await
            .expect("unable to start transaction");
        let value = transaction.get(key).await.expect("unable to get");
        transaction.commit();
        value.unwrap()
    }
}

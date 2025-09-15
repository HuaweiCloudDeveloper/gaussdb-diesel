use anyhow::Result;
use tokio::sync::oneshot;
use diesel_gaussdb::GaussDBConnection;

/// 数据库连接管理器
///
/// 这个管理器在单独的线程中运行，避免tokio运行时冲突
pub struct DatabaseManager {
    db_url: String,
}

impl DatabaseManager {
    pub fn new(db_url: String) -> Self {
        Self { db_url }
    }

    /// 在专用线程中执行数据库操作
    pub async fn execute_query<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&mut GaussDBConnection) -> Result<R, diesel::result::Error> + Send + 'static,
        R: Send + 'static,
    {
        let db_url = self.db_url.clone();
        let (tx, rx) = oneshot::channel();

        // 在专用的阻塞线程中执行数据库操作
        std::thread::spawn(move || {
            let result = (|| -> Result<R, diesel::result::Error> {
                let mut conn = GaussDBConnection::establish(&db_url)
                    .map_err(|e| diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UnableToSendCommand,
                        Box::new(format!("Connection error: {}", e))
                    ))?;
                operation(&mut conn)
            })();

            let _ = tx.send(result);
        });

        rx.await
            .map_err(|_| anyhow::anyhow!("Database operation failed"))?
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))
    }
}

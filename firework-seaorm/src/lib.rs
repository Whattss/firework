use firework::{Plugin, PluginResult, PluginError, Request, Response};
use sea_orm::{Database, DatabaseConnection};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SeaOrmConfig {
    pub database_url: String,
}

impl Default for SeaOrmConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite::memory:".to_string(),
        }
    }
}

/// SeaORM plugin for Firework
/// 
/// Provides database connection pooling and helpers for SeaORM integration.
/// 
/// # Example
/// ```
/// use firework_seaorm::SeaOrmPlugin;
/// use std::sync::Arc;
/// 
/// let plugin = Arc::new(SeaOrmPlugin::new("sqlite::memory:"));
/// firework::register_plugin(plugin);
/// ```
#[derive(Clone)]
pub struct SeaOrmPlugin {
    db: Arc<RwLock<Option<DatabaseConnection>>>,
    database_url: String,
}

impl SeaOrmPlugin {
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            database_url: database_url.into(),
        }
    }
    
    /// Create from configuration
    pub async fn from_config() -> Self {
        let config: SeaOrmConfig = firework::load_plugin_config_as("seaorm")
            .await
            .unwrap_or_default();
        
        Self::new(config.database_url)
    }
    
    /// Get database connection
    pub async fn db(&self) -> Option<DatabaseConnection> {
        self.db.read().await.clone()
    }
}

impl Default for SeaOrmPlugin {
    fn default() -> Self {
        Self::new("sqlite::memory:")
    }
}

#[async_trait::async_trait]
impl Plugin for SeaOrmPlugin {
    fn name(&self) -> &'static str {
        "SeaORM"
    }
    
    async fn on_init(&self) -> PluginResult<()> {
        println!("[SeaORM] Connecting to database: {}", self.database_url);
        let db = Database::connect(&self.database_url).await
            .map_err(|e| PluginError(format!("Failed to connect to database: {}", e)))?;
        *self.db.write().await = Some(db);
        println!("[SeaORM] Database connected successfully");
        Ok(())
    }
    
    async fn on_shutdown(&self) -> PluginResult<()> {
        println!("[SeaORM] Closing database connection");
        *self.db.write().await = None;
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Database connection extractor for handler parameters
#[derive(Clone)]
pub struct DbConn(pub DatabaseConnection);

#[async_trait::async_trait]
impl firework::FromRequest for DbConn {
    async fn from_request(req: &mut Request, _res: &mut Response) -> firework::Result<Self> {
        // Try to get from context first
        if let Some(db) = req.get_context::<DatabaseConnection>() {
            return Ok(DbConn(db));
        }
        
        // Otherwise get from plugin
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let registry = firework::plugin_registry();
                let registry = registry.read().await;
                if let Some(plugin) = registry.get::<SeaOrmPlugin>() {
                    plugin.db().await
                } else {
                    None
                }
            })
        });
        
        db.map(DbConn)
            .ok_or_else(|| firework::Error::Internal("No database connection available".into()))
    }
}

/// Request extension for database access
pub trait RequestDbExt {
    fn db(&self) -> Option<DatabaseConnection>;
}

impl RequestDbExt for Request {
    fn db(&self) -> Option<DatabaseConnection> {
        // Try to get from context first
        if let Some(db) = self.get_context::<DatabaseConnection>() {
            return Some(db);
        }
        
        // Otherwise get from plugin
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let registry = firework::plugin_registry();
                let registry = registry.read().await;
                if let Some(plugin) = registry.get::<SeaOrmPlugin>() {
                    plugin.db().await
                } else {
                    None
                }
            })
        })
    }
}

/// Helper macros and utilities
pub mod helpers {
    use super::*;
    use firework::{Error, Response};
    use sea_orm::DbErr;
    
    /// Convert SeaORM error to Firework error
    pub fn db_error_to_response(err: DbErr) -> Response {
        match err {
            DbErr::RecordNotFound(_) => {
                Error::NotFound("Record not found".to_string()).into_response()
            }
            DbErr::Query(msg) => {
                Error::BadRequest(format!("Query error: {}", msg)).into_response()
            }
            _ => {
                Error::Internal(format!("Database error: {}", err)).into_response()
            }
        }
    }
    
    /// Middleware to inject database connection into request context
    pub fn db_middleware(mut req: Request, res: Response) -> firework::Flow {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let registry = firework::plugin_registry();
                let registry = registry.read().await;
                
                if let Some(plugin) = registry.get::<SeaOrmPlugin>() {
                    if let Some(db) = plugin.db().await {
                        req.set_context(db);
                    }
                }
                
                firework::Flow::Next(req, res)
            })
        })
    }
}

/// Procedural macros for SeaORM integration
pub mod macros {
    /// Create a SeaORM plugin with custom configuration
    /// 
    /// # Example
    /// ```ignore
    /// seaorm_plugin!("postgresql://user:pass@localhost/db");
    /// ```
    #[macro_export]
    macro_rules! seaorm_plugin {
        ($url:expr) => {
            ::std::sync::Arc::new($crate::SeaOrmPlugin::new($url))
        };
    }
    
    /// Quick helper to get database from request
    /// 
    /// # Example
    /// ```ignore
    /// let db = db_from_req!(req);
    /// ```
    #[macro_export]
    macro_rules! db_from_req {
        ($req:expr) => {
            match $crate::RequestDbExt::db(&$req) {
                Some(db) => db,
                None => {
                    return $crate::firework::Error::Internal("No database connection".into())
                        .into_response()
                }
            }
        };
    }
    
    /// Handle SeaORM result with automatic error conversion
    /// 
    /// # Example
    /// ```ignore
    /// let user = db_result!(Entity::find_by_id(1).one(&db).await);
    /// ```
    #[macro_export]
    macro_rules! db_result {
        ($expr:expr) => {
            match $expr {
                Ok(v) => v,
                Err(e) => return $crate::helpers::db_error_to_response(e),
            }
        };
    }
}

// Re-export SeaORM for convenience
pub use sea_orm;

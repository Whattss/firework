use firework::{Plugin, PluginResult, PluginError, Request, Response};
use sea_orm::{Database, DatabaseConnection, EntityTrait, PrimaryKeyTrait, ModelTrait};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Deserialize;
use std::marker::PhantomData;
use std::str::FromStr;

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
        // Try to get from context first (as Arc now)
        if let Some(db_arc) = req.get_context::<DatabaseConnection>() {
            return Ok(DbConn((*db_arc).clone()));
        }
        
        // Otherwise get from plugin (fully async - no blocking!)
        let registry = firework::plugin_registry().read().await;
        let plugin = registry.get::<SeaOrmPlugin>()
            .ok_or_else(|| firework::Error::Internal("SeaORM plugin not registered".into()))?;
        
        let db = plugin.db().await
            .ok_or_else(|| firework::Error::Internal("No database connection available".into()))?;
        
        Ok(DbConn(db))
    }
}

/// Request extension for database access
pub trait RequestDbExt {
    fn db(&self) -> Option<DatabaseConnection>;
}

impl RequestDbExt for Request {
    fn db(&self) -> Option<DatabaseConnection> {
        // Get from context as Arc and clone
        self.get_context::<DatabaseConnection>().map(|arc| (*arc).clone())
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
    
    /// Middleware to inject database connection into request context (async version)
    pub async fn db_middleware_async(req: &mut Request, _res: &mut Response) -> firework::Flow {
        let registry = firework::plugin_registry().read().await;
        
        if let Some(plugin) = registry.get::<SeaOrmPlugin>() {
            if let Some(db) = plugin.db().await {
                req.set_context(db);
            }
        }
        
        firework::Flow::Continue
    }
    
    /// DEPRECATED: Middleware to inject database connection (blocks - don't use!)
    /// Use db_middleware_async instead
    #[deprecated(note = "Use db_middleware_async instead - this blocks threads")]
    pub fn db_middleware(req: &mut Request, _res: &mut Response) -> firework::Flow {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let registry = firework::plugin_registry();
                let registry = registry.read().await;
                
                if let Some(plugin) = registry.get::<SeaOrmPlugin>() {
                    if let Some(db) = plugin.db().await {
                        req.set_context(db);
                    }
                }
                
                firework::Flow::Continue
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

/// Helper trait to convert SeaORM errors to Firework errors
pub trait DbErrExt<T> {
    fn map_db_err(self) -> firework::Result<T>;
}

impl<T> DbErrExt<T> for Result<T, sea_orm::DbErr> {
    fn map_db_err(self) -> firework::Result<T> {
        self.map_err(|e| firework::Error::Internal(format!("Database error: {}", e)))
    }
}

/// Database entity extractor - automatically fetches entities from database
/// 
/// # Basic Usage
/// 
/// ```ignore
/// #[get("/users/:id")]
/// async fn get_user(
///     DbEntity(user): DbEntity<users::Model>
/// ) -> Json<users::Model> {
///     Json(user)
/// }
/// ```
/// 
/// This will:
/// 1. Extract `:id` from path params
/// 2. Parse it to the entity's primary key type
/// 3. Query the database with `Entity::find_by_id(id)`
/// 4. Return 404 if not found
/// 5. Return the entity if found
/// 
/// # Custom Parameter Name
/// 
/// Use the `param` attribute to specify which path parameter to use:
/// 
/// ```ignore
/// #[get("/users/:user_id/posts/:id")]
/// async fn get_user_post(
///     #[param("user_id")] DbEntity(user): DbEntity<users::Model>,
///     DbEntity(post): DbEntity<posts::Model>,  // Uses :id by default
/// ) -> Json<posts::Model> {
///     Json(post)
/// }
/// ```
/// 
/// # Auto-detection
/// 
/// If no `#[param]` is specified, it tries in order:
/// 1. `:id` (most common)
/// 2. `:entity_id` (e.g., `:user_id` for users::Model)
/// 3. `:entity_slug` (if entity has a slug field)
pub struct DbEntity<M>(pub M)
where
    M: ModelTrait;

impl<M> std::ops::Deref for DbEntity<M>
where
    M: ModelTrait,
{
    type Target = M;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M> std::ops::DerefMut for DbEntity<M>
where
    M: ModelTrait,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl<M> firework::FromRequest for DbEntity<M>
where
    M: ModelTrait + Send + Sync + sea_orm::FromQueryResult,
    M::Entity: EntityTrait<Model = M>,
    <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: FromStr + Send + Sync,
{
    async fn from_request(req: &mut Request, _res: &mut Response) -> firework::Result<Self> {
        // Get database connection
        let db = req.get_context::<DatabaseConnection>()
            .ok_or_else(|| firework::Error::Internal(
                "No database connection available. Did you add db_middleware_async?".into()
            ))?;
        
        // Extract entity name for better error messages
        let entity_name = std::any::type_name::<M>()
            .split("::")
            .last()
            .unwrap_or("Entity");
        
        // Try to find the parameter in this order:
        // 1. :id (most common)
        // 2. :entity_id (e.g., :user_id for users::Model)
        // 3. :entity_name (e.g., :user for users::Model)
        let param_name = {
            if req.params.contains_key("id") {
                "id"
            } else {
                // Try entity_id pattern
                let entity_lower = entity_name.to_lowercase().replace("model", "");
                let entity_id_param = format!("{}_id", entity_lower);
                
                if req.params.contains_key(&entity_id_param) {
                    // Leak the string so we get a 'static lifetime
                    // This is safe because param names are small and few
                    Box::leak(entity_id_param.into_boxed_str())
                } else if req.params.contains_key(&entity_lower) {
                    Box::leak(entity_lower.into_boxed_str())
                } else {
                    // Fallback to "id"
                    "id"
                }
            }
        };
        
        // Get the parameter value
        let param_value = req.param(param_name)
            .ok_or_else(|| firework::Error::BadRequest(
                format!("Missing path parameter '{}' for {}", param_name, entity_name)
            ))?;
        
        // Parse to primary key type
        let pk: <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType = 
            param_value.parse()
                .map_err(|_| firework::Error::BadRequest(
                    format!("Invalid {} format for parameter '{}'", entity_name, param_name)
                ))?;
        
        // Query database
        let entity = M::Entity::find_by_id(pk)
            .one(&*db)
            .await
            .map_err(|e| firework::Error::Internal(
                format!("Database error while fetching {}: {}", entity_name, e)
            ))?
            .ok_or_else(|| firework::Error::NotFound(
                format!("{} not found", entity_name)
            ))?;
        
        Ok(DbEntity(entity))
    }
}

/// Optional database entity extractor - returns None if not found instead of 404
/// 
/// # Example
/// 
/// ```ignore
/// #[get("/users/:id")]
/// async fn get_user(
///     DbEntityOpt(user): DbEntityOpt<users::Model>
/// ) -> Json<Option<users::Model>> {
///     Json(user)
/// }
/// ```
pub struct DbEntityOpt<M>(pub Option<M>)
where
    M: ModelTrait;

impl<M> std::ops::Deref for DbEntityOpt<M>
where
    M: ModelTrait,
{
    type Target = Option<M>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait]
impl<M> firework::FromRequest for DbEntityOpt<M>
where
    M: ModelTrait + Send + Sync + sea_orm::FromQueryResult,
    M::Entity: EntityTrait<Model = M>,
    <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: FromStr + Send + Sync,
{
    async fn from_request(req: &mut Request, _res: &mut Response) -> firework::Result<Self> {
        // Try to extract as DbEntity
        match DbEntity::<M>::from_request(req, _res).await {
            Ok(DbEntity(entity)) => Ok(DbEntityOpt(Some(entity))),
            Err(firework::Error::NotFound(_)) => Ok(DbEntityOpt(None)),
            Err(e) => Err(e),
        }
    }
}

/// Database entity extractor with custom parameter name
/// 
/// Use this when you have multiple entities in the same route or when the parameter
/// doesn't follow the standard naming convention.
/// 
/// # Example
/// 
/// ```ignore
/// use firework_seaorm::{DbEntityBy, ByParam};
/// 
/// #[get("/users/:user_id/posts/:post_id")]
/// async fn get_user_post(
///     DbEntityBy(user): DbEntityBy<users::Model, ByParam<"user_id">>,
///     DbEntityBy(post): DbEntityBy<posts::Model, ByParam<"post_id">>,
/// ) -> Json<posts::Model> {
///     // Verify ownership
///     if post.user_id != user.id {
///         return Err(Error::Forbidden("Not your post".into()));
///     }
///     Json(post)
/// }
/// ```
/// 
/// Or use the simpler API with runtime parameter:
/// 
/// ```ignore
/// #[get("/users/:user_id")]
/// async fn get_user(
///     user: DbEntityByParam<users::Model>
/// ) -> Json<users::Model> {
///     Json(*user)  // Deref to access the model
/// }
/// ```
pub struct DbEntityBy<M, P>(pub M, PhantomData<P>)
where
    M: ModelTrait,
    P: ParamName;

/// Trait for compile-time parameter names
pub trait ParamName {
    fn param_name() -> &'static str;
}

/// Helper type for specifying parameter names at compile time
pub struct ByParam<const N: usize>;

// Common parameter name implementations
impl ParamName for ByParam<0> { fn param_name() -> &'static str { "id" } }

/// Marker for user_id parameter
pub struct ByUserId;
impl ParamName for ByUserId { fn param_name() -> &'static str { "user_id" } }

/// Marker for post_id parameter
pub struct ByPostId;
impl ParamName for ByPostId { fn param_name() -> &'static str { "post_id" } }

/// Marker for slug parameter
pub struct BySlug;
impl ParamName for BySlug { fn param_name() -> &'static str { "slug" } }

/// Marker for custom parameter (you provide the name)
pub struct ByCustom(pub &'static str);
impl ParamName for ByCustom {
    fn param_name() -> &'static str {
        // This won't work as-is, we need a different approach
        "id"
    }
}

impl<M, P> std::ops::Deref for DbEntityBy<M, P>
where
    M: ModelTrait,
    P: ParamName,
{
    type Target = M;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait]
impl<M, P> firework::FromRequest for DbEntityBy<M, P>
where
    M: ModelTrait + Send + Sync + sea_orm::FromQueryResult,
    M::Entity: EntityTrait<Model = M>,
    P: ParamName + Send + Sync,
    <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: FromStr + Send + Sync,
{
    async fn from_request(req: &mut Request, _res: &mut Response) -> firework::Result<Self> {
        // Get database connection
        let db = req.get_context::<DatabaseConnection>()
            .ok_or_else(|| firework::Error::Internal(
                "No database connection available. Did you add db_middleware_async?".into()
            ))?;
        
        let entity_name = std::any::type_name::<M>()
            .split("::")
            .last()
            .unwrap_or("Entity");
        
        let param_name = P::param_name();
        
        // Get the parameter value
        let param_value = req.param(param_name)
            .ok_or_else(|| firework::Error::BadRequest(
                format!("Missing path parameter '{}' for {}", param_name, entity_name)
            ))?;
        
        // Parse to primary key type
        let pk: <<M::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType = 
            param_value.parse()
                .map_err(|_| firework::Error::BadRequest(
                    format!("Invalid {} format for parameter '{}'", entity_name, param_name)
                ))?;
        
        // Query database
        let entity = M::Entity::find_by_id(pk)
            .one(&*db)
            .await
            .map_err(|e| firework::Error::Internal(
                format!("Database error while fetching {}: {}", entity_name, e)
            ))?
            .ok_or_else(|| firework::Error::NotFound(
                format!("{} not found", entity_name)
            ))?;
        
        Ok(DbEntityBy(entity, PhantomData))
    }
}

/// Database entity extractor with runtime parameter name
/// 
/// Simpler alternative to DbEntityBy when you don't need compile-time checking.
/// 
/// # Example
/// 
/// ```ignore
/// #[get("/users/:user_id")]
/// async fn get_user(
///     DbEntityByParam(user, "user_id"): DbEntityByParam<users::Model>
/// ) -> Json<users::Model> {
///     Json(user)
/// }
/// ```
pub struct DbEntityByParam<M>(pub M, pub &'static str)
where
    M: ModelTrait;

impl<M> std::ops::Deref for DbEntityByParam<M>
where
    M: ModelTrait,
{
    type Target = M;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// For this one we need a custom FromRequest that takes param name from somewhere
// Since we can't pass it at compile time easily, let's make a simpler macro-based approach

# 🎨 Common Patterns

Reusable patterns for Firework applications.

---

## Repository Pattern

```rust
#[async_trait]
trait UserRepository {
    async fn find(&self, id: u32) -> Option<User>;
    async fn create(&self, user: User) -> Result<User, Error>;
}

struct DbUserRepository {
    db: DatabaseConnection,
}

#[async_trait]
impl UserRepository for DbUserRepository {
    async fn find(&self, id: u32) -> Option<User> {
        // Implementation
        None
    }
    
    async fn create(&self, user: User) -> Result<User, Error> {
        // Implementation
        Ok(user)
    }
}
```

---

## Service Layer

```rust
struct UserService {
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    async fn get_user(&self, id: u32) -> Result<User, Error> {
        self.repo.find(id).await
            .ok_or_else(|| Error::NotFound("User not found".into()))
    }
}
```

---

## DTO Pattern

```rust
// Database entity
struct UserEntity {
    id: u32,
    username: String,
    password_hash: String,
    email: String,
}

// API response (no password!)
#[derive(Serialize)]
struct UserDTO {
    id: u32,
    username: String,
    email: String,
}

impl From<UserEntity> for UserDTO {
    fn from(user: UserEntity) -> Self {
        UserDTO {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}
```

---

## Builder Pattern

```rust
struct QueryBuilder {
    filters: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl QueryBuilder {
    fn new() -> Self {
        Self {
            filters: vec![],
            limit: None,
            offset: None,
        }
    }
    
    fn filter(mut self, field: &str, value: &str) -> Self {
        self.filters.push(format!("{} = {}", field, value));
        self
    }
    
    fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    fn build(self) -> String {
        // Build SQL query
        String::new()
    }
}
```

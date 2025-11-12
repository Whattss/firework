pub mod users;
pub mod posts;

pub mod prelude {
    pub use super::users::Entity as Users;
    pub use super::posts::Entity as Posts;
}

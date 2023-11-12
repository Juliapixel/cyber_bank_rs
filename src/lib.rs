#![forbid(unsafe_code)]

/// handles all authentication and authorization functions of this application,
/// including user registration and login
pub mod auth;

/// convenience mathods for connecting to and setting up the database
pub mod db;

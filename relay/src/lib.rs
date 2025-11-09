#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] nostr_db::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Notify(#[from] notify::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Str(&'static str),
}

impl actix_web::ResponseError for Error {}

pub type Result<T, E = Error> = core::result::Result<T, E>;

mod app;
pub mod duration;
mod extension;
mod hash;
mod list;
pub mod message;
mod reader;
mod server;
mod session;
pub mod setting;
mod subscriber;
mod writer;

pub use metrics;
pub use nostr_db as db;
pub use {
    app::*, extension::*, list::List, reader::Reader, server::Server, session::Session,
    setting::Setting, subscriber::Subscriber, writer::Writer,
};

#[cfg(test)]
pub fn temp_data_path(p: &str) -> anyhow::Result<tempfile::TempDir> {
    Ok(tempfile::Builder::new()
        .prefix(&format!("nostr-relay-test-db-{}", p))
        .tempdir()?)
}

#[cfg(test)]
pub fn create_test_app(db_path: &str) -> anyhow::Result<App> {
    Ok(App::create(
        None,
        false,
        None,
        Some(temp_data_path(db_path)?),
    )?)
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::ResponseError;

    #[test]
    fn test_error_display() {
        let db_error = Error::Db(nostr_db::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "db error",
        )));
        assert_eq!(format!("{}", db_error), "io: db error");

        let io_error = Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert_eq!(format!("{}", io_error), "file not found");

        let config_error = Error::Config(config::ConfigError::Message("config error".to_string()));
        assert_eq!(format!("{}", config_error), "config error");

        let notify_error = Error::Notify(notify::Error::new(notify::ErrorKind::PathNotFound));
        assert_eq!(format!("{}", notify_error), "No path was found.");

        let json_error = Error::Json(serde_json::from_str::<serde_json::Value>("{").unwrap_err());
        assert!(format!("{}", json_error).contains("EOF while parsing an object"));

        let invalid_error = Error::Invalid("invalid input".to_string());
        assert_eq!(format!("{}", invalid_error), "invalid: invalid input");

        let message_error = Error::Message("custom message".to_string());
        assert_eq!(format!("{}", message_error), "custom message");

        let str_error = Error::Str("static string error");
        assert_eq!(format!("{}", str_error), "static string error");
    }

    #[test]
    fn test_error_response_error() {
        let error = Error::Invalid("test".to_string());
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_temp_data_path() -> anyhow::Result<()> {
        let temp_dir = temp_data_path("test_temp_data_path")?;
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
        Ok(())
    }

    #[actix_web::test]
    async fn test_create_test_app() -> anyhow::Result<()> {
        let _app = create_test_app("test_create_test_app")?;
        // You might want to add more assertions here to check the app's state
        // For now, just ensuring it creates without error is a good start.
        Ok(())
    }
}

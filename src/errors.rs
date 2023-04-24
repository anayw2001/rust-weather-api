use actix_web::{error, http::StatusCode};
pub trait IntoHttpError<T> {
    fn http_error(
        self,
        message: &str,
        status_code: StatusCode,
    ) -> core::result::Result<T, actix_web::Error>;

    fn http_internal_error(self, message: &str) -> core::result::Result<T, actix_web::Error>
    where
        Self: std::marker::Sized,
    {
        self.http_error(message, StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl<T, E: std::fmt::Debug> IntoHttpError<T> for core::result::Result<T, E> {
    fn http_error(
        self,
        message: &str,
        status_code: StatusCode,
    ) -> core::result::Result<T, actix_web::Error> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => {
                tracing::error!("http_error: {:?}", err);
                Err(error::InternalError::new(message.to_string(), status_code).into())
            }
        }
    }
}

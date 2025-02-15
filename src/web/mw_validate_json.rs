use crate::web::Error;
use axum::extract::Request;
use axum::{extract::FromRequest, Json, RequestExt};
use validator::Validate;

pub struct ValidatedJson<J>(pub J);

impl<S, J> FromRequest<S> for ValidatedJson<J>
where
    S: Send + Sync,
    J: Validate + 'static,
    Json<J>: FromRequest<()>,
{
    type Rejection = Error;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = req
            .extract::<Json<J>, _>()
            .await
            .map_err(|_| Error::JsonSchema)?;
        data.validate().map_err(Error::JsonValidation)?;
        Ok(Self(data))
    }
}

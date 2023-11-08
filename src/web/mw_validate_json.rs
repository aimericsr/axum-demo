use crate::web::Error;
use axum::{async_trait, extract::FromRequest, Json, RequestExt};
use hyper::Request;
use validator::Validate;

pub struct ValidatedJson<J>(pub J);

#[async_trait]
impl<S, B, J> FromRequest<S, B> for ValidatedJson<J>
where
    B: Send + 'static,
    S: Send + Sync,
    J: Validate + 'static,
    Json<J>: FromRequest<(), B>,
{
    type Rejection = Error;

    async fn from_request(req: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = req
            .extract::<Json<J>, _>()
            .await
            .map_err(|_| Error::JsonSchema)?;
        data.validate().map_err(|err| Error::JsonValidation(err))?;
        Ok(Self(data))
    }
}

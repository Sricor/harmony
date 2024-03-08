use super::*;

pub mod get {
    use std::sync::Arc;

    use crate::time;

    use super::*;

    #[derive(Debug, Serialize)]
    pub struct ResponseBody {
        state: usize,
        timestamp: i64,
    }

    #[instrument(skip(state), name = "GET General Health")]
    pub async fn request(TractState(state): TractState<AppState>) -> ResponseResult<ResponseBody> {
        let response = Response::ok(ResponseBody {
            state: Arc::strong_count(&state),
            timestamp: time::timestamp_millis(),
        });

        Ok(response)
    }
}

pub mod binance_spot_limit;

use std::error::Error;
use std::fmt::Display;
use std::time::Duration;
use std::{future::Future, pin::Pin, sync::Arc};

use delay::task::{Task, TaskBuilder};

use crate::api::State;
use crate::database::collection::{
    PersonIdentifier, Promise, PromiseBinanceSpotLimitInterface, PromiseCategory,
    PromiseIdentifier, PromiseInterface, PromiseLogging, PromiseLoggingInterface, PromiseRunning,
};
use crate::database::error::RecorderError;
use crate::database::{Database, RecorderResult, Uniquer};

type PinFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;
type PromiseProcess<T> = Box<dyn Fn() -> PinFuture<T> + Send + Sync>;

pub trait Process {
    fn create(self, state: Arc<State>) -> PromiseProcess<()>;
}

#[derive(Debug)]
pub enum ProcessError {
    Pursue(String),
    Person(String),
    Database(String),
}

impl Error for ProcessError {}
impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Pursue(e) => e,
            Self::Person(e) => e,
            Self::Database(e) => e,
        };

        write!(f, "{}", message)
    }
}

pub type ProcessResult<T> = Result<T, ProcessError>;

impl Promise {
    pub fn make(self, process: PromiseProcess<()>) -> Arc<Task> {
        let data = TaskBuilder::default()
            .set_identifier(self.identifier)
            .set_interval(Duration::from_secs(self.interval))
            .set_timeout(Duration::from_secs(self.timeout))
            .set_max_concurrent(self.max_concurrent)
            .set_process(process)
            .build();

        Arc::new(data)
    }
}

// async fn insert_info(
//     database: &Database,
//     promise: PromiseIdentifier,
//     owner: PersonIdentifier,
//     message: String,
// ) -> RecorderResult<()> {
//     let item = PromiseLogging::with_info(promise, owner, message);

//     database.promise_logging.insert(&item).await
// }

async fn insert_error(
    database: &Database,
    promise: PromiseIdentifier,
    owner: PersonIdentifier,
    message: String,
) -> RecorderResult<()> {
    let item = PromiseLogging::with_error(promise, owner, message);

    database.promise_logging.insert(&item).await
}

impl From<RecorderError> for ProcessError {
    fn from(err: RecorderError) -> Self {
        Self::Database(err.to_string())
    }
}

pub async fn inital_service_promise(state: Arc<State>) {
    let database = state.database();
    let delay = state.delay();

    let promises = database
        .promise
        .select_all_by_running(&PromiseRunning::Running)
        .await
        .unwrap();

    for p in promises.into_iter() {
        let task = match p.category {
            PromiseCategory::BinanceSpotLimit => {
                let item = database
                    .promise_binance_spot_limit
                    .select_one_by_promise(p.identifier())
                    .await
                    .unwrap()
                    .unwrap();
                p.make(item.create(state.clone()))
            }
        };

        delay.insert(task).await.unwrap();
    }
}

pub mod binance_spot_limit;
// pub mod permanent;

use std::error::Error;
use std::fmt::Display;
use std::time::Duration;
use std::{future::Future, pin::Pin, sync::Arc};

use delay::task::{Task, TaskBuilder};
use serde::{Deserialize, Serialize};

use crate::api::State;
use crate::database::collection::{
    PersonIdentifier, Promise, PromiseIdentifier, PromiseInterface, PromiseProcessBinanceSpotLimit,
    PromiseProcessCategory, PromiseProcessStatus,
};
use crate::database::error::RecorderError;

type PinFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;
type ClosureFuture<T> = Box<dyn Fn() -> PinFuture<T> + Send + Sync>;

pub trait Scheduling {
    fn make<T>(self, state: Arc<State>) -> SchedulingResult<Arc<Task>>
    where
        T: Process + Serialize + for<'a> Deserialize<'a>;
}

impl Scheduling for Promise {
    fn make<T>(self, state: Arc<State>) -> SchedulingResult<Arc<Task>>
    where
        T: Process + Serialize + for<'a> Deserialize<'a>,
    {
        let process = self.process::<T>()?;

        let result = TaskBuilder::default()
            .set_identifier(self.identifier.clone())
            .set_interval(Duration::from_secs(self.interval))
            .set_timeout(Duration::from_secs(self.timeout))
            .set_max_concurrent(self.max_concurrent)
            .set_process(process.create(state, self.owner, self.identifier))
            .build();

        Ok(Arc::new(result))
    }
}

pub trait Process {
    fn create(
        self,
        state: Arc<State>,
        owner: PersonIdentifier,
        promise: PromiseIdentifier,
    ) -> ClosureFuture<()>;
}

#[derive(Debug)]
pub enum SchedulingError {
    // Process running error
    Pursue(String),

    // Person config error
    Person(String),

    // Database error
    Database(String),
}

impl Error for SchedulingError {}
impl Display for SchedulingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Pursue(e) => e,
            Self::Person(e) => e,
            Self::Database(e) => e,
        };

        write!(f, "{}", message)
    }
}

impl From<RecorderError> for SchedulingError {
    fn from(err: RecorderError) -> Self {
        Self::Database(err.to_string())
    }
}

type SchedulingResult<T> = Result<T, SchedulingError>;

pub async fn initial_service_promise(state: Arc<State>) {
    let database = state.database();
    let delay = state.delay();

    let promises = database
        .promise
        .select_all_by_status(&PromiseProcessStatus::Running)
        .await
        .unwrap();

    for p in promises.into_iter() {
        let task = match p.category {
            PromiseProcessCategory::BinanceSpotLimit => {
                p.make::<PromiseProcessBinanceSpotLimit>(state.clone())
            }
        };

        delay.insert(task.unwrap()).await.unwrap();
    }
}

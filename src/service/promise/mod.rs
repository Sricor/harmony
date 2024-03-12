pub mod binance_spot_limit;
// pub mod permanent;

use std::error::Error;
use std::fmt::Display;
use std::time::Duration;
use std::{future::Future, pin::Pin, sync::Arc};

use delay::task::{Task, TaskBuilder};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, error::SendError, Sender};

use crate::api::State;
use crate::database::collection::{
    PersonIdentifier, Promise, PromiseIdentifier, PromiseInterface, PromiseLoggingInterface,
    PromiseLoggingLevel, PromiseProcessBinanceSpotLimit, PromiseProcessCategory,
    PromiseProcessStatus,
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
        let identifier = self.identifier.clone();
        let owner = self.owner.clone();
        let process = self.process::<T>()?;

        let (tx, mut rx) = mpsc::channel::<(PromiseLoggingLevel, String)>(8);
        let state_clone = state.clone();
        tokio::spawn(async move {
            while let Some(v) = rx.recv().await {
                state_clone
                    .database()
                    .promise_logging
                    .insert_by_promise_owner_level_message(&identifier, &owner, &v.0, &v.1)
                    .await
                    .unwrap();
            }
        });

        let result = TaskBuilder::default()
            .set_identifier(self.identifier.clone())
            .set_interval(Duration::from_secs(self.interval))
            .set_timeout(Duration::from_secs(self.timeout))
            .set_max_concurrent(self.max_concurrent)
            .set_process(process.create(state, tx, self.owner, self.identifier))
            .build();

        Ok(Arc::new(result))
    }
}

pub trait Process {
    fn create(
        self,
        state: Arc<State>,
        logger: Sender<(PromiseLoggingLevel, String)>,
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

impl<T> From<SendError<T>> for SchedulingError {
    fn from(err: SendError<T>) -> Self {
        Self::Pursue(err.to_string())
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

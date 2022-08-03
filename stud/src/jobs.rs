use std::future::Future;

use anyhow::Result;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, FutureExt, TryFutureExt};

pub type JobFuture = BoxFuture<'static, Result<Outcome>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Outcome {
    pub exit: bool,
}

impl From<()> for Outcome {
    fn from(_: ()) -> Self {
        Self { exit: false }
    }
}

#[derive(Default)]
pub struct Jobs {
    pub(super) jobs: FuturesUnordered<JobFuture>,
}

impl Jobs {
    pub fn spawn<O: Into<Outcome>, F: Future<Output = Result<O>> + Send + 'static>(
        &mut self,
        future: F,
    ) {
        self.jobs.push(future.map_ok(|t| t.into()).boxed());
    }
}

#[cfg(test)]
mod test {
    use futures_util::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_job() {
        let mut jobs = Jobs::default();
        jobs.spawn(async move { anyhow::Ok(()) });

        assert_eq!(
            jobs.jobs.next().await.unwrap().unwrap(),
            Outcome { exit: false },
        );

        jobs.spawn(async move { Ok(Outcome { exit: true }) });
        assert_eq!(
            jobs.jobs.next().await.unwrap().unwrap(),
            Outcome { exit: true },
        );
    }
}

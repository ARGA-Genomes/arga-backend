use std::sync::Arc;

use async_graphql::*;
use async_graphql::extensions::*;
use async_graphql::futures_util::TryFutureExt;


pub struct ErrorLogging;

impl ExtensionFactory for ErrorLogging {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ErrorLoggingExtension::default())
    }
}


#[derive(Default)]
struct ErrorLoggingExtension;

#[async_graphql::async_trait::async_trait]
impl Extension for ErrorLoggingExtension {
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        let fut = next.run(ctx, info).inspect_err(|err| {
            tracing::error!(
                target: "arga_backend::http",
                message = %err,
                error = ?err,
            );
        });

        fut.await
    }
}

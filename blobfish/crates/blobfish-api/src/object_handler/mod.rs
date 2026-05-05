use axum::Router;
use axum::routing::{get, put};
use blobfish_core::object_service::ObjectService;

mod handler;

pub fn router(router: Router<ObjectService>) -> Router<ObjectService> {
    router
        /*.route("/buckets", get(handlers::get_buckets))
        .route( "/buckets/{bucket}"
                ,put(handlers::put_bucket)
                    .delete(crate::buckets_handler::handlers::delete_bucket)
                    .get(crate::buckets_handler::handlers::get_bucket)
        )*/
}
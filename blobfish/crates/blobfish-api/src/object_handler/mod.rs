use axum::Router;
use axum::routing::{get, put};
use blobfish_core::object_service::ObjectService;

mod handler;

pub fn router(router: Router<ObjectService>) -> Router<ObjectService> {
    router
        .route( "/objects/{bucket}/{*key}"
                ,put(handler::put_object)
                    .delete(handler::delete_object)
                    .get(handler::get_object)
                    .head(handler::head_object)
        )
}
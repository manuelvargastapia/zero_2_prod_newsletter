use actix_web::HttpResponse;

/// Endpoint to  verify the application es up and ready.
///
/// **Returns 200 OK with no body**
///
/// It can be used to customize some alert system to get noitified when
/// the API is down. Or trigger a restart in the context of container
/// orchestration when the API has become unresponsive.
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

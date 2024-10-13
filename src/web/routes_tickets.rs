use crate::model::{ModelController, Ticket, TicketCreate};
use crate::Result;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};

pub fn routes(controller: ModelController) -> Router {
    Router::new()
        .route("/tickets", post(create_ticket).get(list_tickets))
        .route("/tickets/:id", delete(delete))
        .with_state(controller)
}

async fn create_ticket(
    State(controller): State<ModelController>,
    Json(ticket): Json<TicketCreate>,
) -> Result<Json<Ticket>> {
    println!("{:<12} - create_ticket", "HANDLER");
    let ticket = controller.create(ticket).await?;
    Ok(Json(ticket))
}

async fn list_tickets(State(controller): State<ModelController>) -> Result<Json<Vec<Ticket>>> {
    println!("{:<12} - list_tickets", "HANDLER");
    let tickets = controller.list().await?;
    Ok(Json(tickets))
}

async fn delete_ticket(
    State(controller): State<ModelController>,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>> {
    println!("{:<15} - delete_ticket", "HANDLER");
    let ticket = controller.delete(id).await?;
    Ok(Json(ticket))
}

use goose::prelude::*;
use goose::{GooseError, GooseAttack, taskset, task};
use isahc::AsyncReadResponseExt;
use isahc::http::StatusCode;
use serde::{Serialize, Deserialize};

#[cfg(test)]
mod tests;

async fn create_tasks(user: &mut GooseUser) -> GooseTaskResult {
    let goose_metric = create_task(user).await?;

    assert_eq!(goose_metric.response.unwrap().status(), StatusCode::CREATED);

    Ok(())
}

async fn create_task(user: &mut GooseUser) -> Result<goose::goose::GooseResponse, GooseTaskError> {
    let json = serde_json::json!({
        "name": "foo",
        "done": false
    });
    let goose_metric = user.post_json("/api/tasks", &json).await?;
    Ok(goose_metric)
}

async fn get_task(user: &mut GooseUser, task_id: String) -> GooseTaskResult {
    let goose_metrics = user.get(&format!("/api/tasks/{}", task_id)).await?;

    assert_eq!(goose_metrics.response.unwrap().status(), StatusCode::OK);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: String,
    name: String,
    done: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    let endpoint = std::env::var("API_ENDPOINT").expect("missing API_ENDPOINT");

    let json = serde_json::json!({
        "name": "foo",
        "done": false
    });

    let json_string = serde_json::to_string(&json).unwrap();

    let task_request = isahc::Request::builder()
        .header("Content-Type", "application/json")
        .uri(format!("{}api/tasks", endpoint))
        .method("POST")
        .body(json_string)
        .unwrap();

    let mut task = isahc::send_async(task_request).await.unwrap();
    let json: Task = task.json().await.unwrap();

    GooseAttack::initialize()?
        .register_taskset(taskset!("Create Tasks")
            .register_task(task!(create_tasks))
        )
        .register_taskset(taskset!("Get Tasks")
            .register_task(GooseTask::new(std::sync::Arc::new(move |s| {
                std::boxed::Box::pin(get_task(s, json.id.clone()))
            })))
        )
        .execute()
        .await?
        .print();

    Ok(())
}

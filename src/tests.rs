use isahc::{ReadResponseExt, http::StatusCode};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://localhost:8080/api";

#[derive(Deserialize, Serialize, Debug)]
enum TaskResponse {
    Task {
        id: String,
        name: String,
        done: Option<bool>,
    },
    TaskError {
        request_id: String,
        error_codes: Vec<String>,
        error_type: String,
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TaskError {
    request_id: String,
    error_codes: Vec<String>,
    error_type: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Task {
    id: String,
    name: String,
    done: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
struct TaskRequest {
    name: Option<String>,
    done: Option<bool>,
}

fn create_task_request(request_json: &str) -> isahc::Request<&str> {
    isahc::Request::builder()
        .header("Content-Type", "application/json")
        .uri(format!("{}/tasks", BASE_URL))
        .method("POST")
        .body(request_json)
        .unwrap()
}

fn parse_as_string<T: serde::ser::Serialize>(request_json: T) -> String {
    let request_json_str = serde_json::to_string(&request_json).unwrap();
    request_json_str
}

fn create_new_task(name: Option<String>, done: Option<bool>) -> Task {
    let request_json = TaskRequest {
        name,
        done,
    };
    let json_string = parse_as_string(request_json);
    let request_json = create_task_request(&json_string);
    let mut response = isahc::send(request_json).unwrap();
    let tasks = response.json::<Task>().unwrap();
    tasks
}

fn get_all_tasks() -> Vec<Task> {
    let mut response = isahc::get(format!("{}/tasks", BASE_URL)).unwrap();
    let tasks = response.json::<Vec<Task>>().unwrap();
    tasks
}

#[test]
fn should_post_task_successfully() {
    let tasks = create_new_task(Some("foo".into()), Some(false));

    assert_eq!(tasks.name, "foo");
    assert_eq!(tasks.done, Some(false));
}

#[test]
fn should_list_all_tasks_successfully() {
    let tasks = get_all_tasks();

    let previous_length = tasks.len();

    create_new_task(Some("foo".into()), Some(false));

    let new_tasks = get_all_tasks();
    let task = new_tasks.last().unwrap();

    assert_eq!(new_tasks.len(), previous_length + 1);
    assert_eq!(task.name, "foo");
    assert_eq!(task.done, Some(false));
}

fn request_new_task_error<T: serde::ser::Serialize>(task: T) -> TaskError {
    let json_string = parse_as_string(task);
    let request_json = create_task_request(&json_string);
    let mut response = isahc::send(request_json).unwrap();
    response.json::<TaskError>().unwrap()
}

#[test]
fn should_return_name_required_error() {
    let task_error = request_new_task_error(TaskRequest {
        name: None,
        done: Some(false),
    });

    assert_eq!(task_error.error_codes, vec!["name_required"]);
}

#[test]
fn should_return_done_required_error() {
    let task_error = request_new_task_error(TaskRequest {
        name: Some("foo".into()),
        done: None
    });

    assert_eq!(task_error.error_codes, vec!["done_required"]);
}

#[test]
fn should_return_name_invalid_error() {
    #[derive(Deserialize, Serialize, Debug)]
    struct Request {
        name: u32,
        done: Option<bool>
    }

    let task_error = request_new_task_error(Request {
        name: 123,
        done: Some(false)
    });

    assert_eq!(task_error.error_codes, vec!["name_invalid"]);
}

#[test]
fn should_return_done_invalid_error() {
    #[derive(Deserialize, Serialize, Debug)]
    struct Request {
        name: Option<String>,
        done: u32
    }

    let task_error = request_new_task_error(Request {
        name: Some("foo".into()),
        done: 123
    });

    assert_eq!(task_error.error_codes, vec!["done_invalid"]);
}

#[test]
fn should_return_json_invalid_error() {
    let request = isahc::Request::builder()
        .header("Content-Type", "application/json")
        .uri(format!("{}/tasks", BASE_URL))
        .method("POST")
        .body("{,,")
        .unwrap();

    let mut response = isahc::send(request).unwrap();

    let task_error = response.json::<TaskError>().unwrap();

    assert_eq!(task_error.error_codes, vec!["json_invalid"]);
}

#[test]
fn should_return_validation_error() {
    let request = isahc::Request::builder()
        .header("Content-Type", "application/json")
        .uri(format!("{}/tasks", BASE_URL))
        .method("POST")
        .body("{ \"name\": \"foo\", \"done\": true, \"extra\": \"foo\" }")
        .unwrap();

    let mut response = isahc::send(request).unwrap();

    let task_error = response.json::<TaskError>().unwrap();

    assert_eq!(task_error.error_codes, vec!["extra_invalid"]);
}

#[test]
fn should_return_get_one_task() {
    let created_task = create_new_task(Some("foo".into()), Some(false));

    let mut response = isahc::get(format!("{}/tasks/{}", BASE_URL, created_task.id)).unwrap();

    let task = response.json::<Task>().unwrap();

    assert_eq!(created_task, task);
}

#[test]
fn should_return_404_response_only() {
    let mut response = isahc::get(format!("{}/tasks/{}", BASE_URL, "foo")).unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "");
}

#[test]
fn should_return_404_response_only_for_unknown_route() {
    let mut response = isahc::get(format!("{}/foo", BASE_URL)).unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), "");
}

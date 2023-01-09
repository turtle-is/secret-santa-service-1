use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, Mutex},
};

use tide::Request;
use crate::Access::Guest;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Access {
    Guest,
    User,
    Admin,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Group {
    name: String,
    creator: String,
    members: Vec<String>,
    admins: Vec<String>,
    closed: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct User {
    name: String,
    access: Access,
    group: String,
    recipient: String,
}

#[derive(serde::Deserialize)]
struct AddGroup {
    name: String,
    creator: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DataBase {
    users: HashMap<String, User>,
    groups: HashMap<String, Group>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let version: &'static str = env!("CARGO_PKG_VERSION");

    let database = match File::open("data.base") {
        Ok(file) => serde_json::from_reader(file).map_err(|err| {
            let err = std::io::Error::from(err);
            std::io::Error::new(
                err.kind(),
                format!("Failed to read from database file. {err}"),
            )
        })?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("Database file not found. Creating one.");

            let file = File::create("data.base").map_err(|err| {
                std::io::Error::new(err.kind(), format!("Failed to create database file. {err}"))
            })?;

            let database = DataBase {
                users: HashMap::new(),
                groups: HashMap::new(),
            };

            serde_json::to_writer(file, &database).map_err(|err| {
                let err = std::io::Error::from(err);
                std::io::Error::new(
                    err.kind(),
                    format!("Failed to write to database file. {err}"),
                )
            })?;

            database
        }
        Err(err) => {
            panic!("Failed to open database file. {err}");
        }
    };

    let state = Arc::new(Mutex::new(database));

    let mut app = tide::with_state(state);
    app.at("/version").get(move |_| async move { Ok(serde_json::json!({ "version": version })) });

    app.at("/add-user")
        .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let name: String = request.body_json().await?; // <--------------------- bruh

            eprintln!("Adding user {name}");

            let state = request.state();
            let mut guard = state.lock().unwrap();

            let name2 = name.clone();

            guard.users.insert(name2, User { name, access: Guest, group: "".to_string(), recipient: "".to_string() });

            Ok(tide::StatusCode::Ok)
        });

    app.at("/get-user")
        .get(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let name: String = request.body_json().await?;

            let state = request.state();
            let guard = state.lock().unwrap();

            eprintln!("Searching for user {name}");

            match guard.users.get(&name) {
                None => Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("User {name} not found"),
                )),
                Some(user) => Ok(serde_json::json!({"access": user.access, "group": user.group, "recipient": user.recipient})),
            }
        });

        //------------TO REWORK BELOW
       /*  app.at("/set-admin")
        .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let name: String = request.body_json().await?;

            let state = request.state();
            let guard = state.lock().unwrap();

            eprintln!("Searching for user {name}");
            match guard.users.get(&name){
                None => Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("User {name} not found"),
                )),
                Some(&user) => user.access = Access::Admin,//???
            }
            Ok(tide::StatusCode::Ok)
        });*/

    app.listen("127.0.0.1:8080").await
}
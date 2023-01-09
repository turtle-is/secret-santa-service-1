use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, Mutex},
};
use std::borrow::Borrow;
use std::ops::Deref;

use tide::Request;
use crate::Access::Guest;

#[derive(PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Access {
    Guest,
    User,
    Admin,
}

//Request structures

#[derive(serde::Serialize, serde::Deserialize)]
struct AddGroup {
    group_name: String,
    creator_name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserJoin {
    group_name: String,
    admin_name: String,
    user_name: String,
}

//Data structures

#[derive(Clone)]
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
            let name: String = request.body_json().await?;

            eprintln!("Adding user {name}");

            let state = request.state();
            let mut guard = state.lock().unwrap();

            let name2 = name.clone();

            guard.users.insert(name2, User { name, access: Guest, group: "".to_string(), recipient: "".to_string() });

            Ok(tide::StatusCode::Ok)
        });

    app.at("/delete-user")
        .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let name: String = request.body_json().await?;

            eprintln!("Trying to delete user {name}");

            let state = request.state();
            let mut guard = state.lock().unwrap();

            match guard.users.remove(&name) {
                Some(user) => Ok(tide::StatusCode::Ok),
                None => Err(tide::Error::from_str(tide::StatusCode::NotFound, format!("User not found")))
            }
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

    app.at("/get-group")
        .get(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let group: String = request.body_json().await?;

            let state = request.state();
            let guard = state.lock().unwrap();

            eprintln!("Searching for group {group}");
            match guard.groups.get(&group) {
                None => Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("Group {group} not found"),
                )),
                Some(gr) => Ok(serde_json::json!({"creator": gr.creator, "member": gr.members, "admins": gr.admins})),
            }
        });

    app.at("/add-group")
        .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let AddGroup { group_name, creator_name } = request.body_json().await?; // <--------------------- bruh

            eprintln!("{creator_name} trying to create group {group_name}");

            let state = request.state();
            let mut guard = state.lock().unwrap();

            let group_exist: bool;
            let creator_exist: bool;

            match guard.users.get(&creator_name) {
                Some(user) => creator_exist = true,
                None => return Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("User {creator_name} not found"),
                )),
            }

            match guard.groups.get(&group_name) {
                None => group_exist = false,
                Some(gr) => return Err(tide::Error::from_str(
                    tide::StatusCode::Conflict,
                    format!("Group {group_name} already exists."))),
            }

            let creatorCPY1 = creator_name.clone();
            let creatorCPY2 = creator_name.clone();
            let creatorCPY3 = creator_name.clone();
            let groupCPY1 = group_name.clone();
            let groupCPY2 = group_name.clone();

            guard.groups.insert(groupCPY1,
                                Group {
                                    name: group_name,
                                    creator: creator_name,
                                    members: vec![creatorCPY1],
                                    admins: vec![creatorCPY2],
                                    closed: false,
                                });

            let usr = guard.users.remove(&creatorCPY3).unwrap();
            guard.users.insert(usr.name.clone(), User { name: usr.name, access: Access::Admin, group: groupCPY2, recipient: "".to_string() });

            Ok(tide::StatusCode::Ok)
        });

    app.at("/user-join")
        .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let UserJoin { group_name, admin_name, user_name } = request.body_json().await?; // <--------------------- bruh

            eprintln!("{admin_name} trying to add user {user_name} into group {group_name}");

            let state = request.state();
            let mut guard = state.lock().unwrap();

            let group_exist: bool;
            let admin_exist: bool;
            let user_exist: bool;

            let group_cpy1 = group_name.clone();
            let group_cpy3 = group_name.clone();
            let user_cpy1 = user_name.clone();
            let user_cpy2 = user_name.clone();

            match guard.users.get(&admin_name) {
                Some(admin) => {
                    admin_exist = true;
                    if (admin.group.to_string() != group_name) | (admin.access != Access::Admin) {
                        return Err(tide::Error::from_str(
                            tide::StatusCode::NotFound,
                            format!("User {admin_name} is not admin of this group.")));
                    }
                }
                None => return Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("User {admin_name} not found."),
                )),
            }

            match guard.users.get(&user_name) {
                Some(user) =>
                    user_exist = true,
                None => return Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("User {user_name} not found."),
                )),
            }

            match guard.groups.get(&group_name) {
                Some(group) => {
                    group_exist = true;
                    if group.closed == true {
                        return Err(tide::Error::from_str(
                            tide::StatusCode::Forbidden,
                            format!("Group {group_name} closed."),
                        ));
                    }
                }
                None => return Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("Group {group_name} not found."),
                )),
            }

            let mut gr = guard.groups.remove(&group_name).unwrap();
            gr.members.push(user_cpy1);
            guard.groups.insert(group_cpy1, gr);

            let usr = guard.users.remove(&user_name).unwrap();
            guard.users.insert(user_cpy2, User { name: usr.name, access: Access::User, group: group_cpy3, recipient: "".to_string() });

            Ok(tide::StatusCode::Ok)
        });

    app.at("/start")
        .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
            let group_name: String = request.body_json().await?;

            eprintln!("Starting santa in group {group_name}...");

            let state = request.state();
            let mut guard = state.lock().unwrap();

            match guard.groups.get(&group_name) {
                Some(group) => {
                    if group.closed == true {
                        return Err(tide::Error::from_str(
                            tide::StatusCode::Forbidden,
                            format!("Santa in {group_name} already started."),
                        ));
                    }
                }
                None => return Err(tide::Error::from_str(
                    tide::StatusCode::NotFound,
                    format!("Group {group_name} not found."),
                )),
            }

            let mut gr = guard.groups.remove(&group_name).unwrap();
            gr.closed = true;
            guard.groups.insert(group_name.clone(), gr.clone());

            let mut members = gr.members;

            for i in 0..members.len() - 1 {
                let name = members[i].clone();
                let mut usr = guard.users.remove(&name).unwrap();
                usr.recipient = members[i + 1].clone();
                guard.users.insert(name, usr);
            }

            let name = members.last().clone().unwrap();
            let mut usr = guard.users.remove(name).unwrap();
            usr.recipient = members[0].clone();
            guard.users.insert(name.to_string(), usr);

            eprintln!("Santa in group {group_name} started.");

            Ok(tide::StatusCode::Ok)
        });

    //------------TODO  Rework BELOW
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
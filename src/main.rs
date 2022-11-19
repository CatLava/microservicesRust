use hyper::rt::Future;
use hyper::server::Server;
use hyper::service::NewService;
use hyper::{Error, Method, StatusCode, Request, Response};
use hyper::Body;
use hyper::service::service_fn;
use futures::{future};
use std::sync::{Arc, Mutex};
use slab::Slab;
use std::fmt;
use lazy_static::lazy_static;
use regex::Regex;

// need user and database functions 
// adds shared state to the function
type UserId = u64;
struct UserData;
impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{}")
    }
}
type UserDb = Arc<Mutex<Slab<UserData>>>;
// Index page to return on successful option
const INDEX: &'static str = r#"
 <!doctype html>
 <html>
     <head>
         <title>Rust Microservice</title>
     </head>
     <body>
         <h3>Rust Microservice</h3>
     </body>
 </html>
 "#;

lazy_static!{
    static ref INDEX_PATH: Regex = Regex::new("^/(index\\.html?)?$").unwrap();
    static ref USER_PATH: Regex = Regex::new("^/user/((?P<user_id>\\d+?)/?)?$").unwrap();
    static ref USERS_PATH: Regex = Regex::new("^/users/?$").unwrap();
}

fn microservice_handler(req: Request<Body>, user_db: &UserDb) -> impl Future<Item=Response<Body>, Error=Error> {
    let mut users = user_db.lock().unwrap();
    let response = {
        let method = req.method();
        let path = req.uri().path();

        lazy_static!{
            static ref INDEX_PATH: Regex = Regex::new("^/(index\\.html?)?$").unwrap();
            static ref USER_PATH: Regex = Regex::new("^/user/((?P<user_id>\\d+?)/?)?$").unwrap();
            static ref USERS_PATH: Regex = Regex::new("^/users/?$").unwrap();
        }
        println!("HJ");
        if INDEX_PATH.is_match(path) {
            println!("HJERER1");
            if method == &Method::GET {
                Response::new(INDEX.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if USERS_PATH.is_match(path) {
            if method == &Method::GET {
                let list = users.iter()
                    .map(|(id, _)| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                Response::new(list.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }
        } else if let Some(cap) = USER_PATH.captures(path) {
            let user_id = cap.name("user_id").and_then(|m| {
                m.as_str()
                    .parse::<UserId>()
                    .ok()
                    .map(|x| x as usize)
            });
            match (method, user_id) {
                // other branches will be here
                (&Method::POST, None) => {
                    println!("here at post suer");
                    let id = users.insert(UserData);
                    Response::new(id.to_string().into())
                },
                (&Method::POST, Some(_)) => {
                    println!("here at post suer");
                    response_with_code(StatusCode::BAD_REQUEST)
                },
                (&Method::GET, Some(id)) => {
                    if let Some(data) = users.get(id) {
                        Response::new(data.to_string().into())
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                },
                // Put provides ability to modify the data of user
                (&Method::PUT, Some(id)) => {
                    if let Some(user) = users.get_mut(id) {
                        // use * dereference to change data in storage
                        *user = UserData;
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                },
                (&Method::DELETE, Some(id)) => {
                    if users.contains(id) {
                        users.remove(id);
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                },
                _ => { 
                    response_with_code(StatusCode::METHOD_NOT_ALLOWED)
                },
            } } else {
                response_with_code(StatusCode::NOT_FOUND)
            }
    };
    future::ok(response)
}

fn response_with_code(status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap()
}
fn main() {
    println!("Hello, world!");
    let addr = ([127,0,0,1], 8080).into();
    let builder = Server::bind(&addr);
    let user_db = Arc::new(Mutex::new(Slab::new()));
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| microservice_handler(req, &user_db))
});
    // drop an error from the server
    let server = server.map_err(drop);

    hyper::rt::run(server);
}

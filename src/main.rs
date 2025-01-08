use rocket::fairing::Fairing;

#[macro_use]
extern crate rocket;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "api",
            kind: rocket::fairing::Kind::Response
        }
    }

    async fn on_response<'r>(&self, _req: &'r rocket::Request<'_>, response: &mut rocket::Response<'r>) {
        use rocket::http::Header;
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    }
}

#[derive(Responder)]
enum ServerResponse {
    #[response(status = 200)]
    Bytes(Vec<u8>),
    #[response(status = 202)]
    Error(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 500)]
    InternalServerError(Option<String>)
}

#[post("/", data = "<data>")]
async fn index<'a>(data: &'a str) -> ServerResponse {
    let client = reqwest::Client::builder()
        .build()
        .unwrap();
    let mut acc = vec![];
    let Some(url) = data.lines().last() else {
        return ServerResponse::BadRequest(data.to_string());
    };
    let mut request = client.request(reqwest::Method::GET, url);
    for line in data.lines() {
        if line == "Url" {
            break;
        }
        let Some(idx) = line.find(":") else {
            return ServerResponse::BadRequest(line.to_string());
        };
        request = request.header(&line[0..idx], &line[idx+1..]);
    }
    let Ok(request) = request.build() else{
        return ServerResponse::InternalServerError(None)
    };
    match client.execute(request).await {
        Ok(response) => {
            if !response.status().is_success() {
                return ServerResponse::Error(format!("{:?} {}", response.status(), url.to_string()));
            }
            if let Ok(bytes) = response.bytes().await {
                acc.push(bytes.to_vec());
            }
        },
        Err(e) => {
            return ServerResponse::Error(format!("{:?} {}", e.status(), url.to_string()))
        }
    }
    ServerResponse::Bytes(acc.concat())
}

#[launch]
fn rocket() -> _ {
    rocket::build().configure(rocket::config::Config::figment()).attach(CORS).mount("/", routes![index])
}

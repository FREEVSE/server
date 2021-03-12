use std::{env, convert::Infallible, rc::Rc, net::SocketAddr};
use serde::{Serialize, Deserialize};
use warp::Filter;
use semver::Version;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use tokio::fs::File;
use tokio::prelude::*;

use futures::future;

mod manifest;

use manifest::{Manifest, Binary};


#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    //Load up the manifest
    let man = Manifest::new(String::from("binaries/manifest.json"));
    // GET /hello/warp => 200 OK with body "Hello, warp!"

    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    // GET /check => 200 OK with list of available binary versions
    let check = warp::path("updates")
        .and(warp::query::<ClientVersions>())
        .and_then(move |vers: ClientVersions| {
            let man = man.clone();
            async move {
                let hwv = Version::parse(&vers.hwv).unwrap();
                let fwv = Version::parse(&vers.fwv).unwrap();
                Result::<warp::reply::Json, Infallible>::Ok(warp::reply::json(&man.get_available_binaries(hwv, fwv).await.last().unwrap()))
            }
        });

    let get_update = warp::path!("updates" / String)
        .and_then(move |ver: String| {
            async move {
                let mut bin = File::open(format!("binaries/{}", ver)).await.unwrap();
                let mut contents = vec![];
                bin.read_to_end(&mut contents).await.unwrap();

                Result::<warp::reply::WithHeader<Vec<u8>>, warp::Rejection>::Ok(warp::reply::with_header(contents, "content-type", "binary/octet-stream"))
            }
        });

    println!("The current directory is {}", env::current_dir().unwrap().display());

    let logger = warp::log::custom(|info| {
        log::info!(
            "{} {} {}",
            info.method(),
            info.path(),
            info.status(),
        );
    });

    let routes = warp::get()
        .and(hello.or(check).or(get_update))
        .with(logger);

    warp::serve(routes)
        .tls()
        .cert_path(if cfg!(debug_assertions) {"certs/certificate.crt"} else {"/etc/letsencrypt/live/freevse.org/fullchain.pem"})
        .key_path(if cfg!(debug_assertions) {"certs/privateKey.key"} else {"/etc/letsencrypt/live/freevse.org/privkey.pem"})
        .run(([0, 0, 0, 0], 443))
        .await;
}

async fn get_available_binaries(man: &Manifest, hwv: Version, fwv: Version) -> Result<impl warp::Reply, Infallible>{
    Ok(warp::reply::json(&man.get_available_binaries(hwv, fwv).await))
}

#[derive(Debug)]
enum ServiceError{
    IoError(io::Error)
}
impl warp::reject::Reject for ServiceError {}
impl std::convert::From<std::io::Error> for ServiceError {
    fn from(error: io::Error) -> Self{
        ServiceError::IoError(error)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientVersions{
    hwv: String,
    fwv: String
}
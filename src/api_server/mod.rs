mod start_worker;
mod stop_worker;
mod get_art;
mod add_post;

use std::sync::Mutex;
use worker::WorkerManager;
use db::Mongodb;
use images::size::{Size3000x3000, Size30x30};
use util::IdHashMap;
use self::get_art::MosaicArtResponse;

type OriginImageSize = Size3000x3000;
type PieceImageSize = Size30x30;

pub fn run(mongodb: Mongodb) {
    let cors = ::rocket_cors::Cors::default();
    ::rocket::ignite()
        .manage(Mutex::new(
            WorkerManager::<OriginImageSize, PieceImageSize>::new(mongodb),
        ))
        .manage(Mutex::new(IdHashMap::<MosaicArtResponse>::new()))
        .mount(
            "/",
            routes![
                start_worker::handler,
                get_art::handler,
                stop_worker::handler,
                add_post::handler,
            ],
        )
        .attach(cors)
        .launch();
}

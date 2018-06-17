use std::sync::{Arc, Mutex};
use rocket::{State, response::status::NotFound};
use rocket_contrib::Json;

use images::Image;
use mosaic::MosaicArtId;
use super::{CurrentMosaicArtContainer, CurrentSharedMosaicArt};

// =================================
// get mosaic art API
// =================================

#[get("/<id>")]
fn handler(
    id: u64,
    arts: State<Arc<Mutex<CurrentMosaicArtContainer>>>,
) -> Result<Json<MosaicArtResponse>, NotFound<&'static str>> {
    match arts.inner().lock().unwrap().get(MosaicArtId(id)) {
        Some(ref art) => Ok(Json(construct_response(art))),
        None => Err(NotFound("Nothing is also art...")),
    }
}

fn construct_response(art: &CurrentSharedMosaicArt) -> MosaicArtResponse {
    let mosaic_art = {
        let png_img = art.borrow_image(|img| img.to_png_bytes());
        ::base64::encode(png_img.as_slice())
    };
    let piece_posts = art.borrow_piece_posts(|piece_posts| {
        piece_posts
            .map(|post| {
                let post = post.clone();
                InstaPostResponse {
                    post_id: post.get_id_str().into(),
                    user_name: post.get_username().into(),
                }
            })
            .collect()
    });
    let hashtags = art.borrow_hashtags(|hashtags| hashtags.to_vec());
    MosaicArtResponse {
        mosaic_art: mosaic_art,
        piece_posts: piece_posts,
        insta_hashtags: hashtags,
    }
}

#[derive(Serialize)]
pub struct MosaicArtResponse {
    mosaic_art: String, // base64 encoded,
    piece_posts: Vec<InstaPostResponse>,
    insta_hashtags: Vec<String>,
}

#[derive(Serialize)]
pub struct InstaPostResponse {
    post_id: String,
    user_name: String,
}

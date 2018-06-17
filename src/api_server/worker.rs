use std::{collections::HashSet, sync::{Arc, Mutex}, time::Duration};
use futures::{Future, Stream};

use mosaic::{GrayscalePositionFinder, SharedMosaicArt};
use insta::InstaFeeder;
use images::{Image, size::{MultipleOf, Size, SmallerThan}};
use db::MongodbInstaPost;

const REFRESH_INTERVAL: u64 = 3;

pub struct Worker {
    block_users: Arc<Mutex<HashSet<String>>>,
    insta_feeder: Arc<InstaFeeder>,
    db: Arc<MongodbInstaPost>,
}

impl Worker {
    pub fn new(insta_api_server_host: String, db: MongodbInstaPost) -> Worker {
        Worker {
            block_users: Arc::new(Mutex::new(HashSet::new())),
            insta_feeder: Arc::new(InstaFeeder::new(insta_api_server_host)),
            db: Arc::new(db),
        }
    }

    pub fn add_block_user(&self, user_name: String) {
        self.block_users.lock().unwrap().insert(user_name);
    }

    pub fn run<S, SS, I>(&self, hashtags: Vec<String>, origin_image: I) -> SharedMosaicArt<S, SS>
    where
        S: Size + MultipleOf<SS>,
        SS: Size + SmallerThan<S>,
        I: Image<Size = S>,
    {
        let insta_feeder = self.insta_feeder.clone();
        let block_users = self.block_users.clone();
        let mosaic_art = SharedMosaicArt::new(hashtags.clone());
        let mosaic_art2 = mosaic_art.clone();
        let mosaic_art3 = mosaic_art.clone();
        let mut position_finder = GrayscalePositionFinder::new(origin_image);
        let mongodb = self.db.clone();

        // Initialize mosaic art
        let mut init_posts = self.db.find_by_hashtags(hashtags.as_slice(), 1000);
        for post in init_posts.drain(..) {
            let pos = position_finder.find_position(post.get_image());
            mosaic_art.apply_post(post, pos);
        }

        // The reason why I spawn a new thread is because `tokio::timer` does not work well
        // under multi-threaded environment.
        // https://github.com/tokio-rs/tokio/issues/305
        ::std::thread::spawn(move || {
            let f = insta_feeder
                .run(
                    hashtags,
                    mosaic_art,
                    block_users,
                    Duration::new(REFRESH_INTERVAL, 0),
                )
                .for_each(move |post| {
                    mongodb.insert_one(&post);

                    let pos = position_finder.find_position(post.get_image());
                    mosaic_art2.apply_post(post, pos);
                    Ok(())
                })
                .map_err(|e| error!("{:?}", e));

            ::tokio::run(f)
        });

        mosaic_art3
    }
}

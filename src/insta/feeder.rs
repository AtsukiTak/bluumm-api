use std::{collections::HashSet, sync::{Arc, Mutex}, time::{Duration, Instant}};
use futures::{Future, IntoFuture, Stream, stream::{iter_ok, repeat}};
use tokio::timer::Delay;

use images::{ImageFetcher, size::{MultipleOf, Size, SmallerThan}};
use mosaic::SharedMosaicArt;
use insta::{InstaApi, InstaPost};
use error::Error;

pub struct InstaFeeder {
    insta_api: Arc<InstaApi>,
    image_fetcher: Arc<ImageFetcher>,
}

impl InstaFeeder {
    pub fn new(insta_api_server_host: String) -> InstaFeeder {
        InstaFeeder {
            insta_api: Arc::new(InstaApi::new(insta_api_server_host)),
            image_fetcher: Arc::new(ImageFetcher::new()),
        }
    }

    pub fn run<S, SS>(
        &self,
        hashtags: Vec<String>,
        mosaic_art: SharedMosaicArt<S, SS>,
        block_users: Arc<Mutex<HashSet<String>>>,
        req_interval: Duration,
    ) -> impl Stream<Item = InstaPost<SS>, Error = Error>
    where
        S: Size + MultipleOf<SS>,
        SS: Size + SmallerThan<S>,
    {
        let mut hashtags_cycle = HashtagCycle::new(hashtags);
        let insta_api = self.insta_api.clone();
        let insta_api2 = self.insta_api.clone();
        let image_fetcher = self.image_fetcher.clone();
        let req_interval = req_interval.clone();
        let req_interval2 = req_interval;

        repeat::<_, Error>(0)
            .and_then(move |_| {
                let hashtag = hashtags_cycle.next();
                debug!("Search instagram by hashtag : {}", hashtag);
                call_api(insta_api.get_posts_by_hashtag(hashtag), req_interval)
            })
            .map(|posts| iter_ok::<_, Error>(posts))
            .flatten()
            .filter(move |p| {
                let b = mosaic_art.has_post(&p.post_id);
                debug!("Has post {} : {}", &p.post_id, b);
                !b
            })
            .and_then(move |p| call_api(insta_api2.get_post_by_id(&p.post_id), req_interval2))
            .filter(move |p| !block_users.lock().unwrap().contains(&p.user_name))
            .and_then(move |p| {
                image_fetcher
                    .fetch_image::<SS>(p.image_url.as_str())
                    .into_future()
                    .and_then(|img_fut| img_fut)
                    .map(move |img| InstaPost::new(p.post_id, p.user_name, img))
            })
    }
}

/// Send a request and wait some interval.
fn call_api<F>(api_res_fut: F, req_interval: Duration) -> impl Future<Item = F::Item, Error = Error>
where
    F: Future<Error = Error>,
{
    let req_interval = Delay::new(Instant::now() + req_interval).map_err(Error::from);
    api_res_fut.join(req_interval).map(|(res, _)| res)
}

struct HashtagCycle {
    hashtags: Vec<String>,
    next_idx: usize,
}

impl HashtagCycle {
    pub fn new(hashtags: Vec<String>) -> HashtagCycle {
        HashtagCycle {
            hashtags: hashtags,
            next_idx: 0,
        }
    }

    pub fn next(&mut self) -> &str {
        let hashtag = &self.hashtags[self.next_idx];
        self.next_idx += 1;
        if !(self.next_idx < self.hashtags.len()) {
            self.next_idx = 0;
        }
        hashtag
    }
}
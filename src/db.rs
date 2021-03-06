use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use mongodb::{Client, ThreadedClient, coll::{Collection, options::FindOptions},
              db::ThreadedDatabase};
use bson::{Bson, Document, spec::BinarySubtype};

use images::{Image, Size, SizedImage};
use post::{BluummPost, Hashtag, HashtagList, InstaPost, InstaPostId, Post};

#[derive(Clone)]
pub struct Mongodb {
    insta_post: Arc<Collection>,
    bluumm_post: Arc<Collection>,
}

impl Mongodb {
    pub fn new(host: &str, port: u16, db: &str) -> Mongodb {
        debug!(
            "Create new mongodb client with host({}), port({}), db({})",
            host, port, db
        );
        let client = Client::connect(host, port).expect("Fail to create mongodb client");
        let db = client.db(db);
        Mongodb {
            insta_post: Arc::new(db.collection("insta_post")),
            bluumm_post: Arc::new(db.collection("bluumm_post")),
        }
    }

    pub fn insert_one_insta_post<S: Size>(&self, post: &InstaPost<S>) {
        debug!("Insert new insta post into mongodb");
        let doc = doc! {
            "id": post.post_id.as_str(),
            "username": post.user_name(),
            "image": (BinarySubtype::Generic, post.image().to_png_bytes()),
            "hashtag": post.hashtag().as_str(),
            "inserted_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        self.insta_post
            .insert_one(doc, None)
            .expect("Should delegate this error");
    }

    pub fn contains_insta_post(&self, post_id: &InstaPostId) -> bool {
        let filter = doc! { "id": post_id.as_str() };
        self.insta_post
            .find_one(Some(filter), None)
            .expect("Should handle this error")
            .is_some()
    }

    pub fn find_insta_posts_by_hashtags<S: Size>(
        &self,
        hashtags: &HashtagList,
        limit: i64,
    ) -> Vec<InstaPost<S>> {
        debug!("Find posts by hashtags : {:?}", hashtags);
        let hashtags_filter: Vec<Bson> = hashtags
            .iter()
            .map(|h| bson!(doc!{ "hashtag": h.as_str() }))
            .collect();
        let filter = doc! {
            "$or": hashtags_filter,
        };
        let option = {
            let mut op = FindOptions::new();
            op.limit = Some(limit);
            op.sort = Some(doc!{"inserted_time": -1});
            op
        };
        self.insta_post
            .find(Some(filter), Some(option))
            .expect("Fail to execute find operation")
            .map(|res| doc_2_insta_post(res.expect("Invalid document")))
            .collect()
    }

    pub fn insert_one_bluumm_post<S: Size>(&self, post: &BluummPost<S>) {
        debug!("Insert new bluumm post into mongodb");
        let doc = doc! {
            "username": post.user_name(),
            "image": (BinarySubtype::Generic, post.image().to_png_bytes()),
            "hashtag": post.hashtag().as_str(),
            "inserted_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        self.bluumm_post
            .insert_one(doc, None)
            .expect("Should delegate this error");
    }

    pub fn find_bluumm_posts_by_hashtags<S: Size>(
        &self,
        hashtags: &HashtagList,
        limit: i64,
    ) -> Vec<BluummPost<S>> {
        debug!("Find posts by hashtags : {:?}", hashtags);
        let hashtags_filter: Vec<Bson> = hashtags
            .iter()
            .map(|h| bson!(doc!{ "hashtag": h.as_str() }))
            .collect();
        let filter = doc! {
            "$or": hashtags_filter,
        };
        let option = {
            let mut op = FindOptions::new();
            op.limit = Some(limit);
            op.sort = Some(doc!{"inserted_time": -1});
            op
        };
        self.bluumm_post
            .find(Some(filter), Some(option))
            .expect("Fail to execute find operation")
            .map(|res| doc_2_bluumm_post(res.expect("Invalid document")))
            .collect()
    }
}

fn doc_2_insta_post<S: Size>(doc: Document) -> InstaPost<S> {
    let id = InstaPostId(doc.get_str("id").unwrap().into());
    let image = {
        let binary = doc.get_binary_generic("image").unwrap();
        let image = Image::from_bytes(binary).unwrap();
        SizedImage::with_resize(image)
    };
    let username = doc.get_str("username").unwrap();
    let hashtag = Hashtag::new(doc.get_str("hashtag").unwrap());
    InstaPost::new(id, image, username, hashtag)
}

fn doc_2_bluumm_post<S: Size>(doc: Document) -> BluummPost<S> {
    let image = {
        let binary = doc.get_binary_generic("image").unwrap();
        let image = Image::from_bytes(binary).unwrap();
        SizedImage::with_resize(image)
    };
    let username = doc.get_str("username").unwrap();
    let hashtag = Hashtag::new(doc.get_str("hashtag").unwrap());
    BluummPost::new(image, username, hashtag)
}

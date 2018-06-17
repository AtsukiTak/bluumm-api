use std::{collections::hash_map::{HashMap, Values}, marker::PhantomData, sync::{Arc, Mutex}};
use image::{Rgba, RgbaImage};

use images::{Image, Position, size::{MultipleOf, Size, SmallerThan}};
use insta::{InstaPost, InstaPostId};

#[derive(Debug)]
pub struct MosaicArtImage<S> {
    image: RgbaImage,
    phantom: PhantomData<S>,
}

impl<S: Size> MosaicArtImage<S> {
    /// Create a clear MosaicArt.
    pub fn new() -> MosaicArtImage<S> {
        const CLEAR_PIXEL: Rgba<u8> = Rgba { data: [0, 0, 0, 0] };
        let clear_img = RgbaImage::from_pixel(S::WIDTH, S::HEIGHT, CLEAR_PIXEL);
        MosaicArtImage {
            image: clear_img,
            phantom: PhantomData,
        }
    }
}

impl<S: Size> Image for MosaicArtImage<S> {
    type Size = S;

    fn image(&self) -> &RgbaImage {
        &self.image
    }

    fn image_mut(&mut self) -> &mut RgbaImage {
        &mut self.image
    }
}

#[derive(Debug)]
pub struct MosaicArt<S, SS> {
    image: MosaicArtImage<S>,
    piece_posts: HashMap<InstaPostId, InstaPost<SS>>,
    position_map: HashMap<Position, InstaPostId>,
    hashtags: Vec<String>,
}

impl<S, SS> MosaicArt<S, SS>
where
    S: Size + MultipleOf<SS>,
    SS: Size + SmallerThan<S>,
{
    pub fn new(hashtags: Vec<String>) -> MosaicArt<S, SS> {
        MosaicArt {
            image: MosaicArtImage::new(),
            piece_posts: HashMap::new(),
            position_map: HashMap::new(),
            hashtags: hashtags,
        }
    }

    pub fn has_post(&self, post_id: &InstaPostId) -> bool {
        self.piece_posts.contains_key(post_id)
    }

    pub fn apply_post(&mut self, post: InstaPost<SS>, pos: Position) {
        debug!("Apply image to {:?}", pos);
        self.image.overpaint_by(post.get_image(), pos.clone());
        if let Some(overrided_post_id) = self.position_map.insert(pos, post.get_id().clone()) {
            self.piece_posts.remove(&overrided_post_id);
        }
        self.piece_posts.insert(post.get_id().clone(), post);
    }

    pub fn get_image(&self) -> &MosaicArtImage<S> {
        &self.image
    }

    pub fn get_hashtags(&self) -> &[String] {
        self.hashtags.as_slice()
    }

    pub fn get_piece_posts(&self) -> Values<InstaPostId, InstaPost<SS>> {
        self.piece_posts.values()
    }
}

#[derive(Debug, Clone)]
pub struct SharedMosaicArt<S, SS>(Arc<Mutex<MosaicArt<S, SS>>>);

impl<S, SS> SharedMosaicArt<S, SS>
where
    S: Size + MultipleOf<SS>,
    SS: Size + SmallerThan<S>,
{
    pub fn new(hashtags: Vec<String>) -> SharedMosaicArt<S, SS> {
        SharedMosaicArt(Arc::new(Mutex::new(MosaicArt::new(hashtags))))
    }

    pub fn has_post(&self, post_id: &InstaPostId) -> bool {
        self.0.lock().unwrap().has_post(post_id)
    }

    pub fn apply_post(&self, post: InstaPost<SS>, pos: Position) {
        self.0.lock().unwrap().apply_post(post, pos);
    }

    pub fn borrow_image<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&MosaicArtImage<S>) -> T,
    {
        f(self.0.lock().unwrap().get_image())
    }

    pub fn borrow_hashtags<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&[String]) -> T,
    {
        f(self.0.lock().unwrap().get_hashtags())
    }

    pub fn borrow_piece_posts<F, T>(&self, f: F) -> T
    where
        F: FnOnce(Values<InstaPostId, InstaPost<SS>>) -> T,
    {
        f(self.0.lock().unwrap().get_piece_posts())
    }
}

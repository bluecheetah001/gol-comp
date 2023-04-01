use std::cell::RefCell;
use std::num::NonZeroUsize;

use bit_iter::BitIter;
use eframe::egui::{Color32, ColorImage, Context, TextureHandle, TextureId, TextureOptions};
use lru::LruCache;
use node::{Block, Node, Population};

const LRU_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(1 << 12).unwrap();

type ImageCache = LruCache<Node, TextureHandle>;
thread_local! {
    static IMAGE_CACHE: RefCell<ImageCache> = RefCell::new(ImageCache::new(LRU_CACHE_SIZE));
}

pub fn with_image(ctx: &Context, node: &Node, f: impl FnOnce(TextureId)) {
    // ideally cache would account for size of the node
    // but in practice this will only be called for nodes with very small depth (probably never more than 64x64 pixels)
    IMAGE_CACHE.with_borrow_mut(|image_cache| {
        let image = image_cache.get_or_insert(node.clone(), || load_image(ctx, node));
        f(image.id())
    });
}

fn load_image(ctx: &Context, node: &Node) -> TextureHandle {
    let image_width = node.width() as usize;
    let mut pixels = vec![Color32::TRANSPARENT; image_width * image_width];
    fill_image(image_width, &mut pixels, 0, node);
    let image = ColorImage {
        size: [image_width, image_width],
        pixels,
    };
    ctx.load_texture("block_image", image, TextureOptions::NEAREST)
}
fn fill_image(image_width: usize, pixels: &mut Vec<Color32>, index: usize, node: &Node) {
    if node.is_empty() {
        return;
    }
    match node.depth_quad() {
        node::DepthQuad::Leaf(leaf) => {
            fill_image_block(image_width, pixels, index, leaf.nw);
            fill_image_block(image_width, pixels, index + 8, leaf.ne);
            fill_image_block(image_width, pixels, index + 8 * image_width, leaf.sw);
            fill_image_block(image_width, pixels, index + 8 * image_width + 8, leaf.se);
        }
        node::DepthQuad::Inner(_, inner) => {
            let half_width = node.half_width() as usize;
            fill_image(image_width, pixels, index, &inner.nw);
            fill_image(image_width, pixels, index + half_width, &inner.ne);
            fill_image(
                image_width,
                pixels,
                index + half_width * image_width,
                &inner.sw,
            );
            fill_image(
                image_width,
                pixels,
                index + half_width * image_width + half_width,
                &inner.se,
            );
        }
    }
}
fn fill_image_block(image_width: usize, pixels: &mut Vec<Color32>, index: usize, block: Block) {
    let bits: u64 = block.to_rows();
    for i in BitIter::from(bits) {
        let i = 63 - i;
        let i = index + (i / 8) * image_width + (i % 8);
        pixels[i] = Color32::WHITE;
    }
}

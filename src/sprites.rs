use eframe::egui::{TextureId, Vec2};
use eframe::epi::{self, Frame};
use image::{self, GenericImageView};

// Based on code from
// https://github.com/emilk/egui/blob/0.16.0/eframe/examples/image.rs
pub fn load_image_from_bytes(image_data: &[u8], frame: &Frame) -> (TextureId, Vec2) {
    let image = image::load_from_memory(image_data).expect("Failed to load image");
    let image_buffer = image.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image_buffer.into_vec();
    let image = epi::Image::from_rgba_unmultiplied(size, &pixels);

    // Allocate a texture:
    let texture = frame.alloc_texture(image);
    let size = Vec2::new(size[0] as f32, size[1] as f32);
    (texture, size)
}

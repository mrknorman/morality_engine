use bevy::{asset::RenderAssetUsages, image::{CompressedImageFormats, ImageSampler, ImageType}, prelude::*};

const DIGITS_PNG: &[u8] = include_bytes!("content/digits.png");

#[derive(Resource)]
pub struct DigitSheet(pub Handle<Image>);

impl FromWorld for DigitSheet {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let img = Image::from_buffer(
            DIGITS_PNG,
            ImageType::Extension("png"),
            CompressedImageFormats::all(),
            /*is_srgb=*/ true,
            ImageSampler::nearest(),
            RenderAssetUsages::default(),
        ).expect("decode digits spritesheet");
        DigitSheet(images.add(img))
    }
}
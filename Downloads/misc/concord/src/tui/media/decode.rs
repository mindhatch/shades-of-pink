use std::{
    io::Cursor,
    sync::{Arc, OnceLock},
};

use image::{DynamicImage, ImageReader, Limits};
use tokio::{sync::mpsc, task};

use super::preview::ImagePreviewKey;

const MAX_CONCURRENT_MEDIA_IMAGE_DECODES: usize = 2;
pub(super) const MAX_DECODED_IMAGE_WIDTH: u32 = 4096;
pub(super) const MAX_DECODED_IMAGE_HEIGHT: u32 = 4096;
const MAX_DECODED_IMAGE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::tui) enum MediaImageDecodeKey {
    Preview(ImagePreviewKey),
    Avatar(String),
    Emoji(String),
}

pub(in crate::tui) struct MediaImageDecodeJob {
    pub(super) key: MediaImageDecodeKey,
    pub(super) generation: u64,
    pub(super) bytes: Arc<[u8]>,
}

pub(in crate::tui) struct MediaImageDecodeResult {
    pub(in crate::tui) key: MediaImageDecodeKey,
    pub(in crate::tui) generation: u64,
    pub(in crate::tui) result: std::result::Result<DynamicImage, String>,
}

pub(in crate::tui) fn spawn_media_image_decode(
    job: MediaImageDecodeJob,
    tx: mpsc::UnboundedSender<MediaImageDecodeResult>,
) {
    let decode_permits = media_image_decode_permits().clone();
    task::spawn(async move {
        let Ok(_permit) = decode_permits.acquire_owned().await else {
            return;
        };
        if let Ok(result) = task::spawn_blocking(move || decode_media_image(job)).await {
            let _ = tx.send(result);
        }
    });
}

fn decode_media_image(job: MediaImageDecodeJob) -> MediaImageDecodeResult {
    let result = decode_image_bytes(&job.bytes);
    MediaImageDecodeResult {
        key: job.key,
        generation: job.generation,
        result,
    }
}

fn media_image_decode_permits() -> &'static Arc<tokio::sync::Semaphore> {
    static PERMITS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    PERMITS.get_or_init(|| {
        Arc::new(tokio::sync::Semaphore::new(
            MAX_CONCURRENT_MEDIA_IMAGE_DECODES,
        ))
    })
}

pub(in crate::tui) fn decode_image_bytes(
    bytes: &[u8],
) -> std::result::Result<DynamicImage, String> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DECODED_IMAGE_WIDTH);
    limits.max_image_height = Some(MAX_DECODED_IMAGE_HEIGHT);
    limits.max_alloc = Some(MAX_DECODED_IMAGE_BYTES);

    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| format!("decode failed: {error}"))?;
    reader.limits(limits);
    reader
        .decode()
        .map_err(|error| format!("decode failed: {error}"))
}

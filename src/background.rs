use crate::img::{self, SaveBuffer};


// The background of the active area in RGB
// - check for a background_final file
// - check for background<i> sample files and generate
// - fail
// TODO: automate the generation of sample files (different startup / mode change)
pub fn fetch_background() -> Option<Vec<u8>> {
    const BACKGROUND_PREFIX: &str = "./data/background";
    const BACKGROUND_FINAL_SUFFIX: &str = "_final";
    const EXTENSION: &str = ".png";

    let bkg_filename = format!("{}{}{}",
        BACKGROUND_PREFIX,
        BACKGROUND_FINAL_SUFFIX,
        EXTENSION);

    if let Some((mut s, _, _)) = img::load(&bkg_filename) {
        img::bgr_to_rgb(&mut s);
        return Some(s) // cached
    }

    // load samples starting at background0
    let mut samples = vec![];
    let mut width = 0;
    let mut height = 0;

    while let Some((s, w, h)) = img::load(&format!("{}{}{}", BACKGROUND_PREFIX, samples.len(), EXTENSION)) {
        width = w;
        height = h;
        samples.push(s)
    }

    if let Some(mut image) = img::mode(&samples) {
        img::bgr_to_rgb(&mut image);
        image.save(width, height, image::ColorType::Rgb8,
            &format!("{}{}{}", BACKGROUND_PREFIX, BACKGROUND_FINAL_SUFFIX, EXTENSION));
        Some(image)
    } else {
        eprintln!("Unable to load background");
        None
    }
}

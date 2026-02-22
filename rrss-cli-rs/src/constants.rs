use std::collections::HashMap;
pub fn get_platforms() -> HashMap<&'static str, (u32, u32)> {
    let mut m = HashMap::new();
    m.insert("instagram-post", (1080, 1080));
    m.insert("instagram-carousel", (1080, 1350));
    m.insert("instagram-story", (1080, 1920));
    m.insert("facebook-post", (1200, 630));
    m.insert("twitter-post", (1600, 900));
    m.insert("linkedin-post", (1200, 627));
    m.insert("og-image", (1200, 630));
    m
}
//

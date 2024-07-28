use crate::service::{AlbumOnService, ArtistOnService};
// use crate::track::Track;
// use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppleMusic {
    pub id: String,
    pub artists: Vec<ArtistOnService>,
    pub album: AlbumOnService,
    pub url: String,
    pub image: Option<String>,
    pub genres: Vec<String>,
    pub audio_preivew: Option<String>,
}
impl AppleMusic {
    fn grab_public_api_key() -> String {
        String::from("test_key")
    }
}

// #[cfg(test)]
// mod tests {
//     // use crate::apple_music::AppleMusic;

//     #[test]
//     fn test_isrc_three_minute_hero_by_the_selector() {
//         dotenv::dotenv().ok();
//         let results = search_by_isrc("GBAYK8000001");
//         // assert!(t.n > 0)
//     }
// }

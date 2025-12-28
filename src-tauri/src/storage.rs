use native_db::*;
use native_model::native_model;
use native_model::Model;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};
const URL_NAMESPACE: Uuid = uuid!("49be3fd4-a796-4392-9ce8-b7af0d3866f3");

// cover url 或许也可以缓存

pub fn get_uuid_from_url(url: &str) -> Uuid {
    Uuid::new_v5(&URL_NAMESPACE, url.as_bytes())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct TrackDbItem {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    #[primary_key]
    pub id: String,
    pub src: String,
}

static TRACK_MODEL: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<TrackDbItem>().unwrap();
    models
});

mod test {
    use native_db::Builder;

    use crate::storage::{TrackDbItem, TrackDbItemKey};

    #[test]
    fn test_uuid() {
        let url = "https://www.baidu.com";
        let uuid_a = super::get_uuid_from_url(url);
        let uuid_b = super::get_uuid_from_url(url);
        assert_eq!(uuid_a.to_string(), uuid_b.to_string());
    }

    #[test]
    fn test_crud_in_track() {
        // init
        let mut db = Builder::new()
            .create(&super::TRACK_MODEL, "./track.db")
            .unwrap();
        // create
        let item = super::TrackDbItem {
            title: "title".to_string(),
            artist: "artist".to_string(),
            cover_url: "cover_url".to_string(),
            duration: 10.0,
            id: "id".to_string(),
            src: "src".to_string(),
        };
        let rw = db.rw_transaction().unwrap();
        rw.insert(item).unwrap();
        rw.commit().unwrap();
        // read
        let r = db.r_transaction().unwrap();
        let table_result: Vec<_> = r
            .scan()
            .primary::<TrackDbItem>()
            .unwrap()
            .all()
            .unwrap()
            .collect();
        for item in table_result.iter() {
            println!("{:?}", *item);
        }
        // remove
        let rw = db.rw_transaction().unwrap();
        for item in table_result {
            rw.remove(item.unwrap()).unwrap();
        }
        rw.commit().unwrap();
    }
}

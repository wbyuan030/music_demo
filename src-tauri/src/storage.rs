use uuid::{uuid, Uuid};

use crate::types::TrackView;
const URL_NAMESPACE: Uuid = uuid!("49be3fd4-a796-4392-9ce8-b7af0d3866f3");

// cover url 或许也可以缓存

pub fn get_uuid_from_url(url: &str) -> Uuid {
    Uuid::new_v5(&URL_NAMESPACE, url.as_bytes())
}

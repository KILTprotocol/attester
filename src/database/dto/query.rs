#[derive(serde::Deserialize, Clone)]
pub struct Pagination {
    pub offset: Option<[u32; 2]>,
    pub sort: Option<[String; 2]>,
    pub filter: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Query {
    pub range: Option<String>,
    pub sort: Option<String>,
    pub filter: Option<String>,
}

impl From<Query> for Pagination {
    fn from(value: Query) -> Self {
        Pagination {
            offset: value
                .range
                .and_then(|offset| serde_json::from_str::<[u32; 2]>(&offset).ok()),

            sort: value.sort.and_then(|sort| serde_json::from_str(&sort).ok()),
            filter: value
                .filter
                .and_then(|filter| serde_json::from_str(&filter).ok()),
        }
    }
}

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    AddExtensionLayer, Json, Router, routing::delete,
};
use futures::future::TryFutureExt;
use opensearch::{
    http::transport::{BuildError, ConnectionPool, SingleNodeConnectionPool, TransportBuilder},
    indices::IndicesCreateParts,
    DeleteParts, OpenSearch,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Clone, Deserialize, Serialize, Debug)]
struct Onsen {
    #[serde(default)]
    pub id: Option<String>, // ID.
    pub area: String,    // 地域
    pub name: String,    // 施設名/旅館名
    pub address: String, // 住所
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct OnsenList {
    pub took: i32,
    pub onsens: Vec<Onsen>,
}

#[derive(Deserialize, Debug)]
struct DocumentWithSource<S>
where
    S: Serialize,
{
    _id: String,
    _index: String,
    _type: String,
    _source: S,
}

#[derive(Deserialize, Debug)]
struct Document {
    _id: String,
    _index: String,
    _type: String,
}

#[derive(Deserialize, Debug)]
struct SearchResultHits<S>
where
    S: Serialize,
{
    hits: Vec<DocumentWithSource<S>>,
}

/// {
///   "took": 3,
///   "timed_out": false,
///   "_shards": { "total": 1, "successful": 1, "skipped": 0, "failed": 0 },
///   "hits": {
///     "total": { "value": 7, "relation": "eq" },
///     "max_score": 1.0,
///     "hits": [
///       {
///         "_index": "onsen",
///         "_type": "_doc",
///         "_id": "_doc",
///         "_score": 1.0,
///         "_source" : {
///           "id": null,
///           "area": "東鳴子温泉",
///           "name": "初音旅館",
///           "address": "宮城県仙台市"
///         }
///       }
///     ]
///   }
/// }
#[derive(Deserialize, Debug)]
struct SearchResult<S>
where
    S: Serialize,
{
    took: i32,
    hits: SearchResultHits<S>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    query: Option<String>,
}

fn create_opensearch_client(url: Url) -> Result<OpenSearch, BuildError> {
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool).disable_proxy().build()?;
    Ok(OpenSearch::new(transport))
}

type DBConnection = OpenSearch;

async fn index() -> &'static str {
    "Let's try actix-web + opensearch!"
}

// async fn search_onsen(
//     Extension(conn): Extension<ConnectionPool>,
//     query: web::Query<SearchQuery>,
// ) -> impl IntoResponse {
//     println!("search_onsen, query: {:?}", &query);
//     let result = match query.query.as_deref().map(|s| s) {
//         None | Some("") => {
//             conn.get_ref()
//                 .search(SearchParts::Index(&["onsen"]))
//                 .send()
//                 .and_then(|r| async { r.json::<SearchResult<Onsen>>().await })
//                 .await
//         }
//         Some(qs) => {
//             conn.get_ref()
//                 .search(SearchParts::Index(&["onsen"]))
//                 .body(json!({
//                 "query": {
//                 "multi_match": {
//                     "query": qs,
//                     "fields": ["name", "address"]
//                 }
//                 }
//                 }))
//                 .send()
//                 .and_then(|r| async { r.json::<SearchResult<Onsen>>().await })
//                 .await
//         }
//     };
//     match result {
//         Ok(result) => {
//             println!(
//                 "search result, took: {}, hits: {:?}",
//                 &result.took, &result.hits
//             );
//             HttpResponse::Ok().json(OnsenList {
//                 took: result.took,
//                 onsens: result
//                     .hits
//                     .hits
//                     .iter()
//                     .map(|d| Onsen {
//                         id: Some(d._id.clone()),
//                         ..d._source.clone()
//                     })
//                     .collect(),
//             })
//         }
//         Err(e) => {
//             println!("Error in search_onsen: {}", &e);
//             HttpResponse::NotFound().finish()
//         }
//     }
// }

// async fn get_onsen(conn: web::Data<DBConnection>, path: web::Path<OnsenPath>) -> impl IntoResponse {
//     println!("get_onsen id: {}", &path.id);
//     let result = conn
//         .get_ref()
//         .get(GetParts::IndexId("onsen", path.id.as_str()))
//         .send()
//         .and_then(|r| async {
//             r.json::<DocumentWithSource<Onsen>>().await.map(|r| Onsen {
//                 id: Some(r._id),
//                 ..r._source
//             })
//         })
//         .await;
//     match result {
//         Ok(onsen) => Json(&onsen),
//         Err(e) => {
//             println!("Error in get_onsen: {}", &e);
//             HttpResponse::NotFound().finish()
//         }
//     }
// }

// async fn create_onsen(conn: web::Data<DBConnection>, data: web::Json<Onsen>) -> impl IntoResponse {
//     println!("create_onsen, data: {:?}", &data);
//     let mut onsen = data.into_inner();
//     if onsen.id.is_some() {
//         return HttpResponse::BadRequest().finish();
//     }
//     let parts = IndexParts::Index("onsen");
//     println!("elasticsearch url: {}", &parts.clone().url());
//     let result = conn
//         .get_ref()
//         .index(parts)
//         .body(onsen.clone())
//         .send()
//         .and_then(|r| async { r.json::<Document>().await })
//         .await;
//     match result {
//         Ok(result) => {
//             println!("created onsen, result: {:?}", &result);
//             onsen.id = Some(result._id);
//             Json(&onsen)
//         }
//         Err(e) => {
//             println!("failed to create onsen, error: {:?}", &e);
//             HttpResponse::NotFound().finish()
//         }
//     }
// }

// async fn update_onsen(
//     Extension(conn): Extension<DBConnection>,
//     Path(id): Path<String>,
//     Json(data): Json<Onsen>,
// ) -> impl Result<Json<Onsen>, Error> {
//     println!("update_onsen id: {}, data: {:?}", id, &data);
//     let onsen = data.into_inner();
//     if (&onsen.id)
//         .as_ref()
//         .filter(|id| id.to_string() == id)
//         .is_none()
//     {
//         println!("Id must match between url and body data");
//         return HttpResponse::NotFound().finish();
//     }
//     let mut doc = onsen.clone();
//     doc.id = None;
//     let parts = UpdateParts::IndexId("onsen", id.as_str());
//     println!("elasticsearch url: {}", &parts.clone().url());
//     let result = conn
//         .get_ref()
//         .update(parts)
//         .body(json!({ "doc": doc }))
//         .send()
//         .and_then(|r| async { r.text().await })
//         .await;
//     match result {
//         Ok(result) => {
//             println!("updated onsen, result: {:?}", &result);
//             Json(&onsen)
//         }
//         Err(e) => {
//             println!("Error in update_onsen: {}", &e);
//             return HttpResponse::NotFound().finish();
//         }
//     }
// }

async fn delete_onsen(
    Extension(conn): Extension<DBConnection>,
    Path(id): Path<String>,
) -> IntoResponse {
    println!("delete_onsen id: {}", id);
    let result = conn
        .get_ref()
        .delete(DeleteParts::IndexId("onsen", id.as_str()))
        .send()
        .and_then(|r| async { r.json::<Document>().await })
        .await;
    match result {
        Ok(result) => Json(json!({ "id": result._id })),
        Err(e) => {
            println!("Error in delete_onsen: {}", &e);
            HttpResponse::NotFound().finish()
        }
    }
}

async fn start(conn: DBConnection) -> std::io::Result<()> {
    Router::new()
        .route("/", index)
        .route("/:id", delete(delete_onsen))
        .layer(AddExtensionLayer::new(conn))
}

async fn setup_index(client: OpenSearch) {
    let result = client
        .indices()
        .create(IndicesCreateParts::Index("onsen_index"))
        .body(json!({
            "mappings": {
                "properties": {
                    "name": {
                        "type": "text",
                        "analyzer": "kuromoji"
                    },
                    "address": {
                        "type": "text",
                        "analyzer": "kuromoji"
                    }
                }
            }
        }))
        .send()
        .and_then(|r| async { r.text().await })
        .await;
    match result {
        Ok(res) => {
            println!("Successfully created an index: {}", &res)
        }
        Err(e) => {
            println!("Failed to setup index, error: {}", &e)
        }
    }
}

fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let client = Url::parse("http://localhost:9200")
        .map_err(|e| format!("Failed to parse url: {}", &e))
        .and_then(|url| {
            create_opensearch_client(url).map_err(|e| {
                format!(
                    "Failed to create \
                                      elasticsearch client: {}",
                    &e
                )
            })
        });
    match client {
        Err(e) => {
            println!("Failed to parse url: {}", &e);
        }
        Ok(conn) => {
            Runtime::new()
                .expect("")
                .block_on(setup_index(conn.clone()));
            if let Err(e) = start(conn) {
                println!("Failed to start server: {}", &e);
            }
            println!("finished");
        }
    }
}

use std::sync::Arc;
use axum::{extract::State, response::Html, routing::{get, post}, Json, Router};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
  //START: Router
  let app = Router::new()
    .route("/scoreSubmit", post(score_submit))
    .route("/", get(high_scores_html))// <callout id="co.highscorehtml" />
    .route("/highScores", get(high_scores_json))// <callout id="co.highscorejson" />
    .with_state(Arc::new(Mutex::new(HighScoreTable::new())));// <callout id="co.highscore_state" />
  //END: Router

  // run it
  let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
    .await
    .unwrap();
  println!("listening on {}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();
}

//START: HighScoreState
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct HighScoreTable {
  entries: Vec<HighScoreEntry>,// <callout id="co.highscoretable" />
}

impl HighScoreTable {
  fn new() -> Self {
    if std::path::Path::new("high_scores.json").exists() { // <callout id="co.highscoretable.exists" />
      let file = std::fs::File::open("high_scores.json").unwrap();// <callout id="co.highscoretable.open" />
      serde_json::from_reader(file).unwrap()// <callout id="co.highscoretable.fromreader" />
    } else {
      Self { entries: Vec::new() }
    }
  }

  fn add_entry(&mut self, entry: HighScoreEntry) {
    self.entries.push(entry);
    self.entries.sort_by(|a, b| b.score.cmp(&a.score));// <callout id="co.highscoretable.sort" />
    self.entries.truncate(10);// <callout id="co.highscoretable.truncate" />
    self.save();
  }

  fn save(&self) {
    let file = std::fs::File::create("high_scores.json").unwrap();// <callout id="co.highscoretable.create" />
    serde_json::to_writer(file, self).unwrap();// <callout id="co.highscoretable.towriter" />
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct HighScoreEntry {
  name: String,
  score: u32,
}
//END: HighScoreState

//START: HighScoreSubmit
async fn score_submit(
  State(table): State<Arc<Mutex<HighScoreTable>>>,// <callout id="co.highscore_submit_state" />
  high_score: Json<HighScoreEntry>,// <callout id="co.highscore_submit_json_body" />
) {
  let mut lock = table.lock().await;
  lock.add_entry(HighScoreEntry {// <callout id="co.highscore_submit_add" />
    name: high_score.name.clone(),
    score: high_score.score,
  });
}
//END: HighScoreSubmit

//START: HighScoreHtml
async fn high_scores_html(
  State(table): State<Arc<Mutex<HighScoreTable>>>,
) -> Html<String> {
  let mut html = String::from("<h1>High Scores</h1>");
  html.push_str("<table>");
  html.push_str("<tr><th>Name</th><th>Score</th></tr>");
  for entry in &table.lock().await.entries {
    html.push_str("<tr>");
    html.push_str("<td>");
    html.push_str(&entry.name);
    html.push_str("</td>");
    html.push_str("<td>");
    html.push_str(&entry.score.to_string());
    html.push_str("</td>");
    html.push_str("</tr>");
    html.push_str("</table>");

  }
  Html(html)
}
//END: HighScoreHtml

//START: HighScoreJson
async fn high_scores_json(
  State(table): State<Arc<Mutex<HighScoreTable>>>,
) -> Json<HighScoreTable> {
  let lock = table.lock().await;
  let table = lock.clone();
  Json(table)
}
//END: HighScoreJson
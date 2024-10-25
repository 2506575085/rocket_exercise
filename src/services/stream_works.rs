use std::collections::HashMap;
use rocket::response::stream::TextStream;
use rocket::serde::json::{self, Value};
use rocket::tokio::time::{self, Duration};
use crate::services::maze_builder::MazeBuilder;
#[get("/hello")]
pub fn hello() -> TextStream![&'static str] {
    TextStream! {
        for word in ["hello", "world"] {
            yield word;
        }
    }
}

#[get("/byte")]
pub fn byte() -> TextStream![String] {
    TextStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        for i in 0..10 {
            yield json::json!(HashMap::from([("value".to_string(), i)])).to_string();
            interval.tick().await;
        }
    }
}


#[get("/maze?<row_count>")]
pub async fn build_maze(row_count: usize) -> TextStream![String] {
    let maze_builder = MazeBuilder::new(row_count);
    TextStream! {
        // let mut interval = time::interval(Duration::from_millis(1));
        for clear_wall in maze_builder {
            if let Some(clear_wall) = clear_wall {
                yield clear_wall.clone().to_json() + "\n";
                // interval.tick().await;
            }
        }
    }
}

#[get("/maze_full?<row_count>")]
pub fn build_maze_full(row_count: usize) -> Value {
    let maze_builder = MazeBuilder::new(row_count);
    maze_builder.get_json_maze()
}

#[cfg(test)]
mod tests {
    use super::MazeBuilder;

  #[test]
  fn it_works() {
    static ROW_NUM: i32 = 10;
    let maze_builder = MazeBuilder::new(ROW_NUM as usize);
    println!("{:#?}", maze_builder)
  }
}


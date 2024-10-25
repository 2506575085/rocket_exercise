use crate::services::maze_builder::MazeBuilder;

// #[get("/echo?channel")]
// pub fn echo_channel(websocket: ws::WebSocket) -> ws::Channel<'static> {
//     use rocket::futures::{ SinkExt, StreamExt };
//     websocket.channel(move |mut stream| Box::pin(async move {
//         while let Some(message) = stream.next().await {
//             let _ = stream.send(message?).await;
//         }
//         Ok(())
//     }))
// }

// #[get("/echo?stream")]
// fn echo_stream(websocket: ws::WebSocket) -> ws::Stream!['static] {
//     ws::Stream! { websocket =>
//         for await message in websocket {
//             yield message?;
//         }
//     }
// }

// #[get("/echo?compose")]
// fn echo_compose(websocket: ws::WebSocket) -> ws::Stream!['static] {
//     websocket.stream(|io| io)
// }


#[get("/maze")]
pub fn build_maze(websocket: ws::WebSocket) -> ws::Stream!['static] {
    let ws = websocket.config(ws::Config {
        // max_send_queue: Some(5),
        ..Default::default()
    });
    ws::Stream! { ws =>
        for await message in ws {
            // let mut interval = time::interval(Duration::from_millis(1));
            let message_str = message?.to_string();
            println!("message_str: {}", message_str);
            if !message_str.starts_with("maze:") {
                continue;
            }
            let row_count = message_str[5..].parse::<usize>().unwrap();
            let maze_builder = MazeBuilder::new(row_count);
            for clear_wall in maze_builder {
                if let Some(clear_wall) = clear_wall {
                    yield ws::Message::from(clear_wall.clone().to_json());
                }
            }
            yield ws::Message::from("done");
        }
    }
}
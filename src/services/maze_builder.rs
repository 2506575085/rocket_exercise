use rand::seq::IteratorRandom as _;
use rocket::serde::json::{self, Value};
use std::collections::{HashMap,HashSet};

#[derive(Debug, Clone)]
enum WallStatus {
    Open,
    Closed,
    Out
}
impl WallStatus {
    fn to_bool(&self) -> bool {
        match self {
            WallStatus::Open => false,
            _ => true
        }
    }
}
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct Maze {
    horizontal: Vec<Vec<WallStatus>>,
    vertical: Vec<Vec<WallStatus>>,
}
impl Maze {
    fn new(row_count: usize) -> Self {
        let mut horizontal = Vec::new();
        let mut vertical = Vec::new();
        let new_row = |out_flag: bool| -> Vec<WallStatus> {
            let row_content = if out_flag { WallStatus::Out } else { WallStatus::Closed };
            vec![row_content; row_count]
        };
        for i in 0..row_count + 1 {
            vertical.push(new_row(i==0 || i==row_count));
            horizontal.push(new_row(i==0 || i==row_count));
        }
        Self {
            horizontal,
            vertical
        }
    }
}
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct Room {
    x: i32,
    y: i32,
}
impl Room {
    fn new(x: i32, y: i32) -> Self {
        Self {x, y}
    }
}
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Wall {
    x: i32,
    y: i32,
    direction: Direction
}
impl Wall {
    fn new(x: i32, y: i32, direction: Direction) -> Self {
        Self {x, y, direction}
    }
    pub fn to_json(&self) -> String {
        json::json!({
          "x": self.x,
          "y": self.y,
          "direction": match self.direction {
            Direction::Horizontal => "h".to_string(),
            Direction::Vertical => "v".to_string()
          }
        }).to_string()
    }
}

#[derive(Debug)]
/// ```
/// h(v)0 x(y) -->
/// y(x) __ __ __ __ __ __
/// |   |__|__|__|__|__|__|
/// v   |__|__|__|__|__|__|
///     |__|__|__|__|__|__|
///     |__|__|__|__|__|__|
///     |__|__|__|__|__|__|
///     |__|__|__|__|__|__|
/// room(2,1) -> wall(2,1,h) wall(2,2,h) wall(1,2,v) wall(1,3,v)
/// ```
pub struct MazeBuilder {
    maze: Maze,
    linked_room_set: HashSet<Room>,
    todo_wall_set: HashSet<Wall>,
    removed_wall_set: HashSet<Wall>,
    start_point: (i32, i32),
    row_count: i32
}

impl MazeBuilder {
    pub fn new(row_count: usize) -> Self {
        let maze = Maze::new(row_count);
        let mut new_maze_build = Self {
            maze,
            linked_room_set: HashSet::new(),
            todo_wall_set: HashSet::new(),
            removed_wall_set: HashSet::new(),
            start_point: (0, 0),
            row_count: row_count as i32,
        };
        Self::init(&mut new_maze_build);
        new_maze_build
    }
    fn init(&mut self) {
        let row_count = self.row_count as usize;
        let start_x = self.start_point.0 as usize;
        let start_y = self.start_point.1 as usize;
        self.maze.horizontal[start_x][start_y] = WallStatus::Open;
        self.maze.horizontal[row_count][row_count - 1] = WallStatus::Open;
        let start_room = Room::new(start_x as i32, start_y as i32);
        self.add_wall_into_todo_set(&start_room);
        self.linked_room_set.insert(start_room);
    }
    // fn prim(&mut self) {
    //     while let Some(random_wall) = self.todo_wall_set.iter().choose(&mut rand::thread_rng()) {
    //         let random_wall = random_wall.clone();
    //         let mut link_num = 0;
    //         let mut unlink_index = 0;
    //         let rooms = Self::get_rooms_by_wall(&random_wall);
    //         for (index, room) in rooms.iter().enumerate() {
    //             if self.linked_room_set.contains(room) {
    //                 link_num += 1;
    //             } else {
    //                 unlink_index = index;
    //             }
    //         }
    //         if link_num != 2 {
    //             let unlink_room = rooms[unlink_index];
    //             self.linked_room_set.insert(unlink_room.clone());
    //             self.add_wall_into_todo_set(&unlink_room);
    //             let Wall { x, y, direction } = random_wall;
    //             let x = x as usize;
    //             let y = y as usize;
    //             match direction {
    //                 Direction::Horizontal => {
    //                     self.maze.horizontal[y][x] = WallStatus::Open;
    //                 }
    //                 Direction::Vertical => {
    //                     self.maze.vertical[y][x] = WallStatus::Open;
    //                 }
    //             }
    //         }
    //         self.removed_wall_set.insert(random_wall.clone());
    //         self.todo_wall_set.remove(&random_wall);
    //     }
    // }
    fn add_wall_into_todo_set(&mut self, room: &Room) {
        let wall_by_room = Self::get_wall_by_room(room);
        wall_by_room.into_iter().for_each(|wall| {
            if wall.y > 0 && wall.y < self.row_count && !self.removed_wall_set.contains(&wall) {
                self.todo_wall_set.insert(wall);
            }
        });
    }
    fn get_wall_by_room(room: &Room) -> Vec<Wall> {
        let &Room {x, y} = room;
        vec![
            Wall::new(x, y, Direction::Horizontal),
            Wall::new(y, x+1, Direction::Vertical),
            Wall::new(x, y+1, Direction::Horizontal),
            Wall::new(y, x, Direction::Vertical),
        ]
    }
    fn get_rooms_by_wall(wall:&Wall) -> Vec<Room> {
        let &Wall {x, y, direction} = wall;
        if direction == Direction::Horizontal {
            vec![
                Room::new(x, y-1),
                Room::new(x, y),
            ]
        } else {
            vec![
                Room::new(y-1, x),
                Room::new(y, x),
            ]
        }
    }
    pub fn get_json_maze(&self) -> Value {
        fn data_map(vec: &Vec<Vec<WallStatus>>) -> Vec<Vec<bool>> {
            vec.iter().map(|row| {
                row.iter().map(WallStatus::to_bool).collect()
            }).collect()
        }
        let h = data_map(&self.maze.horizontal);
        let v = data_map(&self.maze.vertical);
        json::json!(HashMap::from([
            ("h", h),
            ("v", v),
        ]))
    }
}

// 将生成maze过程封装为iterator,否则无法分步yield
impl Iterator for MazeBuilder {
    type Item = Option<Wall>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.todo_wall_set.is_empty() {
            return None;
        }
        let random_wall = *self.todo_wall_set.iter().choose(&mut rand::thread_rng()).unwrap();
        self.todo_wall_set.remove(&random_wall);
        self.removed_wall_set.insert(random_wall);
        let mut link_num = 0;
        let mut unlink_index = 0;
        let rooms = Self::get_rooms_by_wall(&random_wall);
        for (index, room) in rooms.iter().enumerate() {
            if self.linked_room_set.contains(room) {
                link_num += 1;
            } else {
                unlink_index = index;
            }
        }
        if link_num != 2 {
            let unlink_room = rooms[unlink_index];
            self.add_wall_into_todo_set(&unlink_room);
            self.linked_room_set.insert(unlink_room);
            let Wall { x, y, direction } = random_wall;
            let x = x as usize;
            let y = y as usize;
            match direction {
                Direction::Horizontal => {
                    self.maze.horizontal[y][x] = WallStatus::Open;
                }
                Direction::Vertical => {
                    self.maze.vertical[y][x] = WallStatus::Open;
                }
            }
            
        }
        if link_num != 2 {
            Some(Some(random_wall))
        } else {
            Some(None)
        }
    }
}

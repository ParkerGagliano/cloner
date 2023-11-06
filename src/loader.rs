extern crate colored;
extern crate crossterm;
use colored::*;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use std::collections::HashMap;
use std::io::{self, Write};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub struct Loader {
    stop_flag: Arc<AtomicBool>,
}
fn increment_array(arr: &mut Vec<i32>) {
    for i in 0..arr.len() {
        if arr[i] + 1 == 7 {
            arr[i] = 0;
        } else {
            arr[i] += 1;
        }
    }
}
impl Loader {
    pub fn new() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        Loader { stop_flag }
    }

    pub fn start(&self) {
        let mut color_map: HashMap<i32, ColoredString> = HashMap::new();
        color_map.insert(0, "*".red());
        color_map.insert(1, "*".blue());
        color_map.insert(2, "*".green());
        color_map.insert(3, "*".yellow());
        color_map.insert(4, "*".magenta());
        color_map.insert(5, "*".cyan());
        color_map.insert(6, "*".white());
        let mut global_arr: Vec<i32> = vec![5, 4, 3, 2, 1];
        let stop_flag = self.stop_flag.clone();

        thread::spawn(move || {
            // Your loader loop here
            // Check the stop_flag within the loop and exit if it's set to true
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                increment_array(&mut global_arr);

                let rainbow_string: String = global_arr
                    .iter()
                    .map(|&i| color_map.get(&i).unwrap().to_string())
                    .collect();

                // Move the cursor to the beginning of the line and print the rainbow string
                print!("\r{}", rainbow_string);
                io::stdout().flush().unwrap();

                std::thread::sleep(std::time::Duration::from_millis(90));
            }
        });
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
    }
}

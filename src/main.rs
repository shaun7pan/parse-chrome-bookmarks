extern crate serde;
extern crate serde_json;

use serde_json::Value;
use std::env;
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    process::{Command, Stdio},
};

#[derive(Debug)]
struct BookmarkItem {
    name: String,
    url: String,
}

fn parse_bookmarks(file_path: &str) -> Result<Vec<BookmarkItem>, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let bookmarks: Value = serde_json::from_str(&contents)?;
    let mut parsed_bookmarks = Vec::new();

    parse_bookmarks_recursive(&bookmarks, &mut parsed_bookmarks);
    Ok(parsed_bookmarks)
}

fn parse_bookmarks_recursive(bookmarks: &Value, result: &mut Vec<BookmarkItem>) {
    let mut stack = vec![];

    if let Some(bookmark_bar) = bookmarks
        .get("roots")
        .and_then(|roots| roots.get("bookmark_bar"))
    {
        stack.push(bookmark_bar)
    };

    while let Some(current) = stack.pop() {
        let children = current.get("children").and_then(|c| c.as_array());

        if let Some(children_array) = children {
            stack.extend(children_array.iter().map(|child| {
                let name = child.get("name").and_then(|n| n.as_str());
                let url = child.get("url").and_then(|u| u.as_str());

                if let (Some(name_str), Some(url_str)) = (name, url) {
                    result.push(BookmarkItem {
                        name: name_str.to_string(),
                        url: url_str.to_string(),
                    });
                }

                child
            }));
        }
    }
}

fn search_bookmarks(bookmarks: &[BookmarkItem]) -> Option<&BookmarkItem> {
    let input = bookmarks
        .iter()
        .map(|b| b.name.clone())
        .collect::<Vec<String>>()
        .join("\n");

    let mut fzf = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start FZF");

    {
        let stdin = fzf.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to FZF stdin");
    }

    let output = fzf.wait_with_output().expect("Failed to read FZF output");

    if output.status.success() {
        let selected_bookmark = String::from_utf8(output.stdout).expect("Invalid UTF8");

        return bookmarks
            .iter()
            .find(|bookmark| bookmark.name == selected_bookmark.trim());
    };

    None
}

fn open_url(url: &str) {
    let _ = Command::new("open").arg(url).spawn();
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file_path = env::var("BOOKMARK_FILE_PATH").unwrap().replace("\\ ", " ");

    let bookmarks = parse_bookmarks(&input_file_path)?;

    if let Some(selected_bookmark) = search_bookmarks(&bookmarks) {
        // println!("Selected Bookmark: {}", selected_bookmark.name);
        // println!("URL: {}", selected_bookmark.url);
        open_url(&selected_bookmark.url);
    } else {
        println!("No bookmarks selected.")
    }

    Ok(())
}

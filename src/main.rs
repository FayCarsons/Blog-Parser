#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use clap::Parser;
use regex::{Captures, Regex};
use std::cmp::Ordering;
use std::{fmt::Write, fs, path::PathBuf};

use serde_json::from_str;
use std::collections::HashMap;

type Attribute<'a> = &'a str;
type Tag<'a> = &'a str;
type ReplacementMap<'a> = HashMap<Tag<'a>, HashMap<Attribute<'a>, &'a str>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Input file or directory
    #[arg(short, long)]
    input: PathBuf,

    #[arg(long = "js")]
    js_path: Option<PathBuf>,

    // Output directory (has to be a directory)
    #[arg(short, long)]
    output: PathBuf,

    #[arg(short, long)]
    replacements: PathBuf,
}

#[derive(Clone, Debug)]
struct InternalPost {
    title: String,
    date: NaiveDate,
    header: String,
    body: String,
}

#[derive(serde::Serialize)]
struct Post {
    title: String,
    date: String,
    header: String,
    body: String,
}

impl From<InternalPost> for Post {
    fn from(
        InternalPost {
            title,
            date,
            header,
            body,
        }: InternalPost,
    ) -> Self {
        Post {
            title,
            date: date.format("%B %d, %Y").to_string(),
            header,
            body,
        }
    }
}

const POST_TYPE: &str =
    "export type Post = {name: string, date: string, header: string, body: string}";

fn replace(repl: &ReplacementMap, html: &str) -> String {
    let re = Regex::new(r"(<(\w+)([^>]*)>)").unwrap();

    let aux = |caps: &Captures| -> String {
        let tag = &caps[2];
        let existing_attrs = &caps[3];

        let attrs = if let Some(m) = repl.get(tag) {
            m.iter()
        } else {
            return caps[0].to_string();
        };

        let attrs = attrs.fold(String::new(), |mut acc, (attr, value)| {
            let _ = write!(acc, r#" {attr}="{value}""#);
            acc
        });

        format!("<{}{}{}>", tag, existing_attrs, attrs)
    };

    re.replace_all(html, aux).into_owned()
}

fn main() -> std::io::Result<()> {
    let Args {
        input,
        js_path,
        output,
        replacements,
    } = Args::parse();
    let mut titles = Vec::<String>::new();
    let mut post_objects = Vec::<InternalPost>::new();

    let md = if input.is_dir() {
        fs::read_dir(input)?
            .map(|entry| {
                let entry = entry.unwrap();
                entry.path()
            })
            .collect::<Vec<PathBuf>>()
    } else {
        vec![input]
    };

    if !output.exists() {
        fs::create_dir(&output)?;
    }

    let json = fs::read_to_string(&replacements).unwrap_or_else(|_| {
        panic!(
            "JSON replacement map {} does not exist",
            replacements.display()
        )
    });
    let replacement_map =
        from_str::<ReplacementMap>(&json).expect("Invalid structure in JSON replacement map: ");

    for path in md {
        let filename = path
            .file_name()
            .unwrap_or_else(|| panic!("Error in file {:?}", path));
        let content = fs::read_to_string(&path)?;
        let (date, rest) = content.split_once('\n').unwrap_or_else(|| {
            panic!(
                "{} does not have a date in the first line",
                filename.to_str().unwrap()
            )
        });
        let date = date.replace("Date: ", "");
        let date = chrono::naive::NaiveDate::parse_from_str(date.trim(), "%B %d, %Y")
            .unwrap_or_else(|_| panic!("Invalid date format in {}", filename.to_str().unwrap()));

        let html = markdown::to_html(rest);
        let with_classses = replace(&replacement_map, &html);

        let (header, body) = with_classses.split_once('\n').unwrap_or_else(|| {
            panic!(
                "Blog {} does not have a title header",
                filename.to_str().unwrap()
            )
        });
        let (header, body) = (header.to_owned(), body.to_owned());

        let title = filename.to_str().unwrap().replace(".md", "");

        let post = InternalPost {
            title,
            date,
            header,
            body,
        };
        post_objects.push(post);
    }

    post_objects.sort_by(|a, b| Ordering::reverse(a.date.cmp(&b.date)));

    for post @ InternalPost { title, .. } in post_objects.iter() {
        titles.push(title.to_string());
        let json = serde_json::to_string_pretty(&Post::from(post.clone()))
            .unwrap_or_else(|_| panic!("Serializaion error in post: {title}"));
        let path = output.join(format!("{title}.json"));
        fs::write(path, json)?
    }

    let titles_arr = serde_json::to_string_pretty(&titles).expect("Invalid title");
    let ts_file = format!("{POST_TYPE}\n\n export const posts: string[] = {titles_arr};");

    let js = if let Some(path) = js_path {
        path
    } else {
        output
    }
    .join("posts.ts");

    fs::write(js, ts_file)
}

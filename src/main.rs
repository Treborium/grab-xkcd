use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use std::time::Duration;
use std::convert::TryInto;

use anyhow::Result;
use clap::Clap;
use serde_derive::{Deserialize, Serialize};
use url::Url;

const BASE_URL: &str = "https://xkcd.com";
const LATEST_COMIC: usize = 0;

struct XkcdClient {
    args: Args,
}

impl XkcdClient {
    fn new(args: Args) -> Self {
        XkcdClient { args }
    }

    fn run(&self) -> Result<()> {
		let url = if let Some(n) = self.args.num {
			format!("{}/{}/info.0.json", BASE_URL, n)
		} else {
			format!("{}/info.0.json", BASE_URL)
		};

        let http_client = reqwest::blocking::ClientBuilder::new()
            .timeout(Duration::from_secs(self.args.timeout))
            .build()?;
        let resp: ComicResponse = http_client.get(&url).send()?.text()?.try_into()?;
        let comic: Comic = resp.into();

        if self.args.save {
            comic.save()?;
        }

        comic.print(self.args.output)?;
        Ok(())
    }
}

/// A utility to grab XKCD comics
#[derive(Clap)]
pub struct Args {
    /// Set a connection timeout
    #[clap(long, short, default_value = "30")]
    pub timeout: u64,
    /// Print output in a format
    #[clap(long, short, arg_enum, default_value = "text")]
    pub output: OutFormat,
    /// The comic to load
    #[clap(long, short)]
    pub num: Option<usize>,
    /// Save image file to current directory
    #[clap(long, short)]
    pub save: bool,
}


#[derive(Clap, Copy, Clone)]
pub enum OutFormat {
    Json,
    Text,
}


#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ComicResponse {
    month: String,
    num: usize,
    link: String,
    year: String,
    news: String,
    safe_title: String,
    transcript: String,
    alt: String,
    img: String,
    title: String,
    day: String,
}

impl TryFrom<String> for ComicResponse {
    type Error = anyhow::Error;
    fn try_from(json: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&json).map_err(|e| e.into())
    }
}

#[derive(Serialize)]
struct Comic {
    title: String,
    num: usize,
    date: String,
    desc: String,
    img_url: String,
}

impl Comic {
    fn print(&self, of: OutFormat) -> Result<()> {
        match of {
            OutFormat::Text => println!("{}", self),
            OutFormat::Json => println!("{}", serde_json::to_string(self)?),
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let url = Url::parse(&*self.img_url)?;
        let img_name = url.path_segments().unwrap().last().unwrap();
        let path = std::env::current_dir()?;
        let path = path.join(img_name);
        let mut file = std::fs::File::create(path)?;

        let body = reqwest::blocking::get(&self.img_url)?;
        file.write_all(&*body.bytes()?).map_err(|e| e.into())
    }
}

impl From<ComicResponse> for Comic {
    fn from(cr: ComicResponse) -> Self {
        Comic {
            title: cr.title,
            num: cr.num,
            date: format!("{}-{}-{}", cr.day, cr.month, cr.year),
            desc: cr.alt,
            img_url: cr.img,
        }
    }
}

impl fmt::Display for Comic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\n\
			Comic No: {}\n\
            Date: {}\n\
            Description: {}\n\
            Image: {}\n",
            self.title, self.num, self.date, self.desc, self.img_url
        )
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let client = XkcdClient::new(args);
    client.run()
}

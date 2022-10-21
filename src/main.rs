mod args;

use args::Args;
use clap::Parser;
use rss::validation::Validate;
use rss::Channel;
use rss::Guid;
use rss::Image;
use rss::Item;
use scraper::Html;
use scraper::Selector;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

const BASE_URL_FILE_PATH: &str = "CNAME";

fn main() {
    let args = Args::parse();

    // Read
    let mut channel = File::open(&args.feed)
        .map(|file| {
            Channel::read_from(BufReader::new(file)).expect("Cannot read channel from feed file.")
        })
        .unwrap_or_default();

    configure_channel(&mut channel, &args);

    for page in args.pages {
        add_item(&mut channel, &page)
    }

    // Validate and write
    channel.validate().expect("Channel failed validation.");
    channel
        .pretty_write_to(
            File::create(&args.feed).expect("Cannot open feed file for writing."),
            b' ',
            2,
        )
        .expect("Cannot write feed file.");
}

fn configure_channel(channel: &mut Channel, args: &Args) {
    if let Some(title) = &args.title {
        channel.set_title(title)
    }
    if channel.title.is_empty() {
        eprintln!("Warning: Empty channel title.");
    }

    if let Some(description) = &args.description {
        channel.set_description(description)
    }
    if channel.description.is_empty() {
        eprintln!("Warning: Empty channel description.");
    }

    if let Some(base_url) = &args.base_url {
        channel.set_link(base_url)
    } else if channel.link.is_empty() {
        let mut base_url = String::new();
        match File::open(BASE_URL_FILE_PATH).and_then(|mut file| file.read_to_string(&mut base_url))
        {
            Ok(_) => channel.set_link(format!("https://{}/", base_url.trim())),
            Err(_) => eprintln!("Warning: Empty channel link."),
        }
    }

    if let Some(language) = &args.language {
        channel.set_language(language.clone())
    }

    channel.set_last_build_date(chrono::Utc::now().to_rfc2822());

    let mut image = Image::default();
    image.set_title(&channel.title);
    image.set_link(&channel.link);
    image.set_url(format!("{}{}", channel.link, args.favicon));
    channel.set_image(image);
}

fn add_item(channel: &mut Channel, page: &str) {
    // Read and parse HTML
    let mut html = String::new();
    File::open(page)
        .unwrap_or_else(|_| panic!("Cannot open page '{}'.", page))
        .read_to_string(&mut html)
        .unwrap_or_else(|_| panic!("Cannot read from page '{}'", page));
    let document = Html::parse_document(&html);

    // Extract data from HTML
    let title = match document
        .select(&Selector::parse("meta[property=\"og:title\"]").unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [title_element] => title_element
            .value()
            .attr("content")
            .expect("Expected og:title meta element to have a content attribute."),
        _ => panic!(
            "Expected exactly one og:title meta element on page '{}'.",
            page
        ),
    };
    let description = match document
        .select(&Selector::parse("meta[property=\"og:description\"]").unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [description_element] => description_element
            .value()
            .attr("content")
            .expect("Expected og:description meta element to have a content attribute."),
        _ => panic!(
            "Expected exactly one og:description meta element on page '{}'.",
            page
        ),
    };
    let item_url = match document
        .select(&Selector::parse("meta[property=\"og:url\"]").unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [url_element] => url_element
            .value()
            .attr("content")
            .expect("Expected og:url meta element to have a content attribute."),
        _ => panic!(
            "Expected exactly one og:url meta element on page '{}'.",
            page
        ),
    };
    let published_date = match document
        .select(&Selector::parse("meta[property=\"article:published_time\"]").unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [published_time_element] => chrono::DateTime::parse_from_rfc3339(
            published_time_element.value().attr("content").expect(
                "Expected article:published_time meta element to have a content attribute.",
            ),
        )
        .unwrap_or_else(|_| panic!("Cannot parse published date on page '{}'.", page)),
        _ => panic!("Expected exactly one article:published_time meta element."),
    };
    let h2_html = match document
        .select(&Selector::parse("h2").unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [h2_element] => h2_element.html(),
        _ => panic!("Expected exactly one h2 element on page '{}'.", page),
    };
    let content = match document
        .select(&Selector::parse("main").unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        [main_element] => main_element.inner_html().replace(&h2_html, ""),
        _ => panic!("Expected exactly one main element on page '{}'.", page),
    };

    // Find existing item
    let existing_item_index = channel.items.iter().position(|existing_item| {
        existing_item
            .guid
            .as_ref()
            .expect("Expected all channel items to have a guid for keeping track.")
            .value
            == item_url
    });

    // Construct item
    let mut item = match existing_item_index {
        Some(index) => channel.items[index].clone(),
        None => Item::default(),
    };
    item.set_title(title.to_owned());
    item.set_description(description.to_owned());
    item.set_link(item_url.to_owned());
    item.set_pub_date(published_date.to_rfc2822());
    item.set_content(content);
    let mut guid = Guid::default();
    guid.set_value(item_url);
    guid.set_permalink(true);
    item.set_guid(guid);

    // Insert or replace item
    match existing_item_index {
        Some(index) => channel.items[index] = item,
        None => channel.items.insert(0, item),
    }
}

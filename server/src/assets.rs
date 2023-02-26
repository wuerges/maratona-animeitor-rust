use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "client/"]
#[prefix = "prefix/"]
struct Asset;

fn main() {
    let index_html = Asset::get("prefix/index.html").unwrap();
    println!("{:?}", std::str::from_utf8(index_html.data.as_ref()));

    for file in Asset::iter() {
        println!("{}", file.as_ref());
    }
}

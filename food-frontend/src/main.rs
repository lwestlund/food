use maud::html;

#[tokio::main]
async fn main() {
    let markup = html! {
        p { "Hello, world!" }
    };
    println!("{}", markup.into_string());
}

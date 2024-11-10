use reqwest::Error;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use tokio;

#[tokio::main]
async fn main() {
    do_crawl_quote().await;
    do_crawl_book().await;
    println!("All tasks finished!");
}

async fn do_crawl_book() {
    let domain = String::from("https://books.toscrape.com");
    let mut start_url = domain.clone() + "/catalogue/page-1.html";
    loop {
        let res = books_crawl(&start_url).await.unwrap();
        if res == "" {
            break;
        }
        start_url = format!("{}/catalogue/{}", domain, res);
        println!("{}", res);
    }
}

async fn do_crawl_quote() {
    let start_url = "https://quotes.toscrape.com";
    // Create a vector to hold all the task handles
    let mut tasks = vec![];
    // Spawn each crawl task concurrently
    for i in 0..10 {
        let url = start_url.to_string();
        tasks.push(tokio::spawn(async move {
            if let Err(e) = quotes_crawl(&url, &i).await {
                eprintln!("Error crawling page {}: {}", i, e);
            }
        }));
    }

    // Wait for all tasks to finish
    for task in tasks {
        task.await.unwrap();
        // println!("task finish");
    }
}

async fn quotes_crawl(url: &str, page_number: &i32) -> Result<(), Error> {
    let url = format!("{}/page/{}/", url, page_number + 1);

    // Asynchronous GET request
    let response = reqwest::get(&url).await?.text().await?;

    // Parse the HTML document
    let document = Document::from(response.as_str());

    // Print quotes from the page
    for node in document.find(Name("div").descendant(Name("span")).and(Class("text"))) {
        println!("Page {}: {}", page_number + 1, node.text());
    }

    Ok(())
}

async fn books_crawl(url: &str) -> Result<String, Error> {
    let req_res = reqwest::get(url).await?.text().await?;
    let document = Document::from(req_res.as_str());
    for node in document.find(Class("product_pod")) {
        let img_url = node.find(Name("img")).next().unwrap().attr("src").unwrap();
        let title = node.find(Name("h3")).next().unwrap().text();
        let price = node.find(Class("price_color")).next().unwrap().text();
        let book = format!(
            "{{\"title\": \"{}\", \"price\": \"{}\", \"imgUrl\": \"{}\"}}",
            title, price, img_url
        );
        println!("{}", book);
    }
    let res = document
        .find(Class("next"))
        .next() // Try to find the "next" page button
        .and_then(|next| next.find(Name("a")).next()) // Try to find the anchor tag
        .and_then(|a| a.attr("href")) // Try to get the "href" attribute
        .unwrap_or("") // If any step is `None`, return an empty string
        .to_string(); // Convert the result to a String
    Ok(res)
}

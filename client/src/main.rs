use service_sdk_rust::{Config, MyServiceClient, model};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8080".into());
    let client = MyServiceClient::new(Config::new(endpoint));
    let created = client
        .create_item(
            model::CreateItemInput::builder()
                .name("Rust client example")
                .description("Created through the generated SDK")
                .build(),
        )
        .await?;
    let item_id = created.item.item_id;
    println!("Created {item_id}");

    let result = exercise(&client, &item_id).await;
    let deleted = client
        .delete_item(model::DeleteItemInput::builder().item_id(&item_id).build())
        .await;
    result?;
    deleted?;
    println!("Deleted {item_id}");
    Ok(())
}

async fn exercise(
    client: &MyServiceClient,
    item_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let fetched = client
        .get_item(model::GetItemInput::builder().item_id(item_id).build())
        .await?;
    println!("Read: {}", fetched.item.name);

    let updated = client
        .update_item(
            model::UpdateItemInput::builder()
                .item_id(item_id)
                .name("Updated by the Rust client")
                .build(),
        )
        .await?;
    println!("Updated: {}", updated.item.name);

    let mut pages =
        client.list_items_pages(model::ListItemsInput::builder().max_results(1).build());
    while let Some(page) = pages.next_page().await {
        for item in page?.items {
            println!("Listed: {} ({})", item.name, item.item_id);
        }
    }
    Ok(())
}

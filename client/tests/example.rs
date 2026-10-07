use std::process::Command;

use service_sdk_rust::{Config, Error, GetItemError, MyServiceClient, model};

#[tokio::test]
#[ignore = "requires a running service; optionally set CRUCIBLE_ENDPOINT"]
async fn example_paginates_and_cleans_up_without_removing_existing_items()
-> Result<(), Box<dyn std::error::Error>> {
    let endpoint =
        std::env::var("CRUCIBLE_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let client = MyServiceClient::new(Config::new(&endpoint));
    let mut existing = Vec::new();
    for index in 0..2 {
        match client
            .create_item(
                model::CreateItemInput::builder()
                    .name(format!("Existing item {index}"))
                    .build(),
            )
            .await
        {
            Ok(created) => existing.push(created.item),
            Err(error) => {
                if let Err(cleanup_error) = cleanup(&client, &existing).await {
                    eprintln!("Seed cleanup failed: {cleanup_error}");
                }
                return Err(error.into());
            }
        }
    }

    let output = Command::new(env!("CARGO_BIN_EXE_crucible-service-client"))
        .arg(&endpoint)
        .output();
    let mut preserved = Vec::new();
    for item in &existing {
        preserved.push(
            client
                .get_item(
                    model::GetItemInput::builder()
                        .item_id(&item.item_id)
                        .build(),
                )
                .await,
        );
    }
    let cleanup_result = cleanup(&client, &existing).await;

    let output = output?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let item_id = stdout
        .lines()
        .next()
        .unwrap()
        .strip_prefix("Created ")
        .unwrap();
    assert!(stdout.contains("Read: Rust client example"), "{stdout}");
    assert!(
        stdout.contains("Updated: Updated by the Rust client"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!("Listed: Updated by the Rust client ({item_id})")),
        "{stdout}"
    );
    assert!(stdout.contains(&format!("Deleted {item_id}")), "{stdout}");
    for (original, fetched) in existing.iter().zip(preserved) {
        assert_eq!(fetched?.item.name, original.name);
        assert!(
            stdout.contains(&format!("Listed: {} ({})", original.name, original.item_id)),
            "{stdout}"
        );
    }
    assert!(matches!(
        client.get_item(model::GetItemInput::builder().item_id(item_id).build()).await,
        Err(Error::Service(error))
            if matches!(error.error, GetItemError::ResourceNotFoundException(_))
    ));
    cleanup_result?;
    Ok(())
}

async fn cleanup(
    client: &MyServiceClient,
    items: &[model::ItemDetails],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut first_error = None;
    for item in items {
        if let Err(error) = client
            .delete_item(
                model::DeleteItemInput::builder()
                    .item_id(&item.item_id)
                    .build(),
            )
            .await
        {
            eprintln!("Could not delete seeded item {}: {error}", item.item_id);
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }
    match first_error {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}

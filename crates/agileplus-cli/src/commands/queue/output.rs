use anyhow::Result;

use agileplus_domain::domain::backlog::BacklogItem;

pub(crate) fn print_backlog_items(items: &[BacklogItem], output: &str) -> Result<()> {
    if output == "json" {
        println!("{}", serde_json::to_string_pretty(items)?);
        return Ok(());
    }

    if items.is_empty() {
        println!("No backlog items found");
        return Ok(());
    }

    for item in items {
        print_backlog_item(item, output)?;
    }
    Ok(())
}

pub(crate) fn print_backlog_item(item: &BacklogItem, output: &str) -> Result<()> {
    if output == "json" {
        println!("{}", serde_json::to_string_pretty(item)?);
        return Ok(());
    }

    println!(
        "#{:>4} [{}] {} ({}, {})",
        item.id.unwrap_or_default(),
        item.intent,
        item.title,
        item.priority,
        item.status,
    );
    if !item.description.is_empty() {
        println!("  {}", item.description);
    }
    println!("  source: {}", item.source);
    if let Some(ref feature_slug) = item.feature_slug {
        println!("  feature: {feature_slug}");
    }
    if !item.tags.is_empty() {
        println!("  tags: {}", item.tags.join(", "));
    }
    Ok(())
}

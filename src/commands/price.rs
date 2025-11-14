use crate::cache::PriceCache;
use crate::prices::get_prices;
use crate::stations::pick_station_local;
use colored::*;

pub fn execute(
    from: &str,
    to: &str,
    travel_class: Option<String>,
    is_return: bool,
    cache: Option<&PriceCache>,
) -> Result<(), Box<dyn std::error::Error>> {
    let station_from = pick_station_local(from)?;
    let station_to = pick_station_local(to)?;

    let class_param = travel_class.as_deref();
    let travel_type = if is_return { Some("return") } else { Some("single") };

    println!(
        "Getting prices from {} to {}",
        station_from.names.long, station_to.names.long,
    );

    let response = get_prices(&station_from, &station_to, class_param, travel_type, cache)?;

    if response.payload.prices.is_empty() {
        println!("No prices found for this route.");
        return Ok(());
    }

    println!();
    for price in &response.payload.prices {
        let total_euros = price.total_price_in_cents as f64 / 100.0;
        let per_adult_euros = price.price_per_adult_in_cents as f64 / 100.0;

        let class_str = match price.travel_class.as_str() {
            "FIRST_CLASS" => "1st class",
            "SECOND_CLASS" => "2nd class",
            _ => &price.travel_class,
        };

        let mut line = format!(
            "€{:.2} - {} ({})",
            total_euros,
            price.display_name.bold(),
            class_str
        );

        if price.is_best_option {
            line = format!("{} {}", line, "⭐ Best option".green());
        }

        println!("{}", line);
        println!("  Per adult: €{:.2}", per_adult_euros);

        if let Some(discount) = price.discount_in_cents {
            if discount > 0 {
                let discount_euros = discount as f64 / 100.0;
                println!("  Discount: €{:.2}", discount_euros);
            }
        }

        if price.discount_type != "NONE" {
            println!("  Discount type: {}", price.discount_type);
        }

        if let Some(operator) = &price.operator_name {
            println!("  Operator: {}", operator);
        }

        println!();
    }

    Ok(())
}

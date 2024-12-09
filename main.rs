use csv::Reader;
use plotters::prelude::*;
use std::error::Error;

/// Represents artist data points for multiple variables.
#[derive(Debug)]
struct ArtistData {
    total_streams: f64,
    solo_streams: f64,
    feature_streams: f64,
    lead_streams: f64,
}

/// Parses the dataset to extract artist data points.
fn parse_artist_data(file_path: &str) -> Result<Vec<ArtistData>, Box<dyn Error>> {
    println!("Reading file from path: {}", file_path);
    let mut reader = Reader::from_path(file_path)?;
    let mut data_points = Vec::new();

    for record in reader.records() {
        let record = record?;

        let total_streams: f64 = record
            .get(1)
            .unwrap_or("0")
            .replace(',', "")
            .parse()
            .unwrap_or_else(|_| 0.0);
        let solo_streams: f64 = record
            .get(3)
            .unwrap_or("0")
            .replace(',', "")
            .parse()
            .unwrap_or_else(|_| 0.0);
        let feature_streams: f64 = record
            .get(5)
            .unwrap_or("0")
            .replace(',', "")
            .parse()
            .unwrap_or_else(|_| 0.0);
        let lead_streams: f64 = record
            .get(4)
            .unwrap_or("0")
            .replace(',', "")
            .parse()
            .unwrap_or_else(|_| 0.0);

        data_points.push(ArtistData {
            total_streams,
            solo_streams,
            feature_streams,
            lead_streams,
        });
    }

    println!("Successfully parsed {} valid records.", data_points.len());
    Ok(data_points)
}

/// Calculates the linear regression line (slope and intercept).
fn calculate_regression(data: &[(f64, f64)]) -> (f64, f64) {
    let n = data.len() as f64;
    let sum_x: f64 = data.iter().map(|(x, _)| *x).sum();
    let sum_y: f64 = data.iter().map(|(_, y)| *y).sum();
    let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
    let sum_xx: f64 = data.iter().map(|(x, _)| x * x).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    (slope, intercept)
}

/// Visualizes the scatter plot with a regression line.
fn visualize_relationship(
    data: &[(f64, f64)],
    slope: f64,
    intercept: f64,
    title: &str,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(file_name, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_x = data.iter().map(|(x, _)| *x).fold(0.0 / 0.0, f64::max);
    let max_y = data.iter().map(|(_, y)| *y).fold(0.0 / 0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..max_x, 0.0..max_y)?;

    chart.configure_mesh().x_desc("X").y_desc("Y").draw()?;

    chart.draw_series(data.iter().map(|(x, y)| Circle::new((*x, *y), 5, RED.filled())))?;

    chart.draw_series(LineSeries::new(
        (0..=max_x as i32).map(|x| {
            let x = x as f64;
            let y = slope * x + intercept;
            (x, y)
        }),
        &BLUE,
    ))?
    .label(format!("y = {:.2}x + {:.2}", slope, intercept))
    .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &BLUE));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    println!("Scatter plot saved to {}", file_name);
    Ok(())
}

/// Main function to process and analyze data.
fn main() {
    let file_path = "artists.csv";

    // Parse dataset
    let data = match parse_artist_data(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing dataset: {}", e);
            return;
        }
    };

    // Prepare data for each relationship
    let solo_data: Vec<(f64, f64)> = data
        .iter()
        .map(|d| (d.solo_streams, d.total_streams))
        .collect();
    let feature_data: Vec<(f64, f64)> = data
        .iter()
        .map(|d| (d.feature_streams, d.total_streams))
        .collect();
    let lead_data: Vec<(f64, f64)> = data
        .iter()
        .map(|d| (d.lead_streams, d.total_streams))
        .collect();

    // Analyze and visualize Solo Streams vs Total Streams
    let (solo_slope, solo_intercept) = calculate_regression(&solo_data);
    println!(
        "Solo Streams Regression: y = {:.2}x + {:.2}",
        solo_slope, solo_intercept
    );
    if let Err(e) = visualize_relationship(
        &solo_data,
        solo_slope,
        solo_intercept,
        "Total Streams vs Solo Streams",
        "solo_relationship.png",
    ) {
        eprintln!("Error generating solo streams plot: {}", e);
    }

    // Analyze and visualize Featured Streams vs Total Streams
    let (feature_slope, feature_intercept) = calculate_regression(&feature_data);
    println!(
        "Featured Streams Regression: y = {:.2}x + {:.2}",
        feature_slope, feature_intercept
    );
    if let Err(e) = visualize_relationship(
        &feature_data,
        feature_slope,
        feature_intercept,
        "Total Streams vs Featured Streams",
        "featured_relationship.png",
    ) {
        eprintln!("Error generating featured streams plot: {}", e);
    }

    // Analyze and visualize Lead Streams vs Total Streams
    let (lead_slope, lead_intercept) = calculate_regression(&lead_data);
    println!(
        "Lead Streams Regression: y = {:.2}x + {:.2}",
        lead_slope, lead_intercept
    );
    if let Err(e) = visualize_relationship(
        &lead_data,
        lead_slope,
        lead_intercept,
        "Total Streams vs Lead Streams",
        "lead_relationship.png",
    ) {
        eprintln!("Error generating lead streams plot: {}", e);
    }
}

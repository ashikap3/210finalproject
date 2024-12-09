use csv::Reader;
use plotters::prelude::*;
use std::error::Error;

#[derive(Debug, PartialEq)]
struct ArtistData {
    total_streams: f64,
    solo_streams: f64,
    feature_streams: f64,
    lead_streams: f64,
}
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

/// Main function
fn main() {
    let file_path = "artists.csv";

    let data = match parse_artist_data(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing dataset: {}", e);
            return;
        }
    };

    let relationships: Vec<(&str, Box<dyn Fn(&ArtistData) -> (f64, f64)>)> = vec![
        (
            "Total Streams vs Solo Streams",
            Box::new(|d: &ArtistData| (d.solo_streams, d.total_streams)),
        ),
        (
            "Total Streams vs Featured Streams",
            Box::new(|d: &ArtistData| (d.feature_streams, d.total_streams)),
        ),
        (
            "Total Streams vs Lead Streams",
            Box::new(|d: &ArtistData| (d.lead_streams, d.total_streams)),
        ),
    ];

    for (title, mapper) in relationships {
        let relationship_data: Vec<(f64, f64)> = data.iter().map(mapper).collect();
        let (slope, intercept) = calculate_regression(&relationship_data);
        println!("{title} Regression: y = {slope:.2}x + {intercept:.2}");
        let file_name = format!("{}.png", title.replace(' ', "_").to_lowercase());

        if let Err(e) = visualize_relationship(&relationship_data, slope, intercept, title, &file_name) {
            eprintln!("Error generating plot for {title}: {e}");
        }
    }
}

///tests for the program.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_artist_data() {
        let test_csv = "Name,Total Streams,Solo Streams,Lead Streams,Feature Streams\n\
                        Artist1,1000,500,300,200\n\
                        Artist2,2000,800,600,400\n";
        let mut rdr = csv::Reader::from_reader(test_csv.as_bytes());
        let data = parse_artist_data("artists.csv").unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(
            data[0],
            ArtistData {
                total_streams: 1000.0,
                solo_streams: 500.0,
                feature_streams: 200.0,
                lead_streams: 300.0
            }
        );
    }

    #[test]
    fn test_calculate_regression() {
        let data = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let (slope, intercept) = calculate_regression(&data);
        assert!((slope - 2.0).abs() < 1e-6);
        assert!((intercept - 0.0).abs() < 1e-6);
    }
}

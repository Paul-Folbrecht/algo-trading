use std::collections::VecDeque;

use chrono::NaiveDate;

pub struct Series {
    pub date: NaiveDate,
    pub open: f32,
    pub close: f32,
    pub high: f32,
    pub low: f32,
    pub volume: f32,
}

// https://www.investopedia.com/terms/s/sma.asp
pub(crate) fn calculate_sma(series: &[Series], period: usize) -> VecDeque<f32> {
    let mut sma_values: VecDeque<f32> = series
        .windows(period)
        .map(|window| window.iter().map(|item| item.close).sum::<f32>() / period as f32)
        .collect();

    for _ in 1..period {
        sma_values.push_front(0.0);
    }

    assert!(sma_values.len() == series.len());

    sma_values
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_calculate_sma() {
        let series = vec![
            Series {
                date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                open: 1.0,
                close: 1.0,
                high: 1.0,
                low: 1.0,
                volume: 1.0,
            },
            Series {
                date: NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
                open: 2.0,
                close: 2.0,
                high: 2.0,
                low: 2.0,
                volume: 2.0,
            },
            Series {
                date: NaiveDate::from_ymd_opt(2021, 1, 3).unwrap(),
                open: 3.0,
                close: 3.0,
                high: 3.0,
                low: 3.0,
                volume: 3.0,
            },
            Series {
                date: NaiveDate::from_ymd_opt(2021, 1, 4).unwrap(),
                open: 4.0,
                close: 4.0,
                high: 4.0,
                low: 4.0,
                volume: 4.0,
            },
            Series {
                date: NaiveDate::from_ymd_opt(2021, 1, 5).unwrap(),
                open: 5.0,
                close: 5.0,
                high: 5.0,
                low: 5.0,
                volume: 5.0,
            },
        ];

        let sma_values = calculate_sma(&series, 3);
        println!("SMA Values: {:?}", sma_values);
        assert_eq!(sma_values.len(), series.len());
        assert_eq!(sma_values[0], 0.0);
        assert_eq!(sma_values[1], 0.0);
        assert_eq!(sma_values[2], 2.0);
        assert_eq!(sma_values[3], 3.0);
        assert_eq!(sma_values[4], 4.0);
    }
}

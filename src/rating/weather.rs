/// Weather effect calculations
/// Implements specialty-specific weather impacts
use super::types::{Specialty, Weather};

/// Calculate the weather impact factor for a player
///
/// Different player specialties perform differently in various weather conditions:
/// - Technical players: better in sunny (+5%), worse in rainy (-5%)
/// - Powerful players: better in rainy (+5%), worse in sunny (-5%)
/// - Quick players: worse in both rainy and sunny (-5%)
/// - Other specialties: no weather effect (factor = 1.0)
pub fn calc_weather(specialty: Specialty, weather: Weather) -> f64 {
    match (weather, specialty) {
        (Weather::Rainy, Specialty::Technical) => 0.95,
        (Weather::Sunny, Specialty::Technical) => 1.05,
        (Weather::Rainy, Specialty::Powerful) => 1.05,
        (Weather::Sunny, Specialty::Powerful) => 0.95,
        (Weather::Rainy, Specialty::Quick) => 0.95,
        (Weather::Sunny, Specialty::Quick) => 0.95,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_technical_weather_effects() {
        assert_eq!(calc_weather(Specialty::Technical, Weather::Sunny), 1.05);
        assert_eq!(calc_weather(Specialty::Technical, Weather::Rainy), 0.95);
        assert_eq!(calc_weather(Specialty::Technical, Weather::Neutral), 1.0);
    }

    #[test]
    fn test_powerful_weather_effects() {
        assert_eq!(calc_weather(Specialty::Powerful, Weather::Rainy), 1.05);
        assert_eq!(calc_weather(Specialty::Powerful, Weather::Sunny), 0.95);
    }

    #[test]
    fn test_quick_weather_effects() {
        assert_eq!(calc_weather(Specialty::Quick, Weather::Rainy), 0.95);
        assert_eq!(calc_weather(Specialty::Quick, Weather::Sunny), 0.95);
    }

    #[test]
    fn test_no_specialty_no_effect() {
        assert_eq!(calc_weather(Specialty::NoSpecialty, Weather::Rainy), 1.0);
        assert_eq!(calc_weather(Specialty::NoSpecialty, Weather::Sunny), 1.0);
    }
}

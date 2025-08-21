use chrono::{Datelike, NaiveDate};
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::sql_types::{Date, Nullable};

#[derive(Debug)]
pub struct DateParseError;

/// A utility module for handling mixed date formats commonly found in the database
pub struct DateParser;

impl DateParser {
    /// Parse a date string that could be in multiple formats:
    /// - DD/MM/YYYY
    /// - YYYY/MM/DD  
    /// - D/M/YY
    /// - D/M/YYYY
    /// - D/MM/YY
    pub fn parse_flexible_date(date_str: &str) -> Result<NaiveDate, DateParseError> {
        // Check the format by counting digits in the year part
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() != 3 {
            return Err(DateParseError);
        }

        // Determine if this is a 2-digit or 4-digit year format
        // Check first part to see if it's a 4-digit year (YYYY/MM/DD)
        if parts[0].len() == 4 {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
                return Ok(date);
            }
        }

        // Check last part to see if it's a 4-digit year (DD/MM/YYYY or D/M/YYYY)
        if parts[2].len() == 4 {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
                return Ok(date);
            }
        }

        // Handle 2-digit years (D/M/YY or DD/MM/YY)
        if parts[2].len() == 2 {
            // Handle 2-digit years specially: DD/MM/YY format
            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%y") {
                // Convert 2-digit years to full years using the common convention:
                // 00-49 -> 2000-2049, 50-99 -> 1950-1999
                // Note: chrono's %y format returns years in range 1970-2069, so we need to adjust
                let parsed_year = date.year();
                let adjusted_year = if parsed_year >= 2000 && parsed_year <= 2049 {
                    // Already in the correct 2000s range
                    parsed_year
                }
                else if parsed_year >= 2050 && parsed_year <= 2069 {
                    // Convert 2050-2069 to 1950-1969
                    parsed_year - 100
                }
                else if parsed_year >= 1970 && parsed_year <= 1999 {
                    // Convert 1970-1999 to 1970-1999 (keep as-is)
                    parsed_year
                }
                else {
                    // Fallback for unexpected years
                    parsed_year
                };

                if let Some(adjusted_date) = NaiveDate::from_ymd_opt(adjusted_year, date.month(), date.day()) {
                    return Ok(adjusted_date);
                }
            }
        }

        // Return a custom parse error for unsupported formats
        Err(DateParseError)
    }

    /// Creates a SQL expression that converts mixed date formats to PostgreSQL dates
    /// This handles the same formats as parse_flexible_date but at the SQL level
    pub fn sql_date_converter() -> SqlLiteral<Nullable<Date>> {
        sql::<Nullable<Date>>(
            "CASE 
                WHEN event_date ~ '^[0-9]{4}/[0-9]{1,2}/[0-9]{1,2}$' THEN to_date(event_date, 'YYYY/MM/DD')
                WHEN event_date ~ '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{4}$' THEN to_date(event_date, 'DD/MM/YYYY')
                WHEN event_date ~ '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{2}$' THEN 
                    CASE 
                        WHEN event_date ~ '^[0-9]{1}/[0-9]{1,2}/[0-9]{2}$' THEN to_date(event_date, 'D/MM/YY')
                        ELSE to_date(event_date, 'D/M/YY')
                    END
                ELSE NULL
             END",
        )
    }

    /// Creates a SQL expression for ordering by parsed dates (non-nullable version)
    pub fn sql_date_order_converter() -> SqlLiteral<Date> {
        sql::<Date>(
            "CASE 
                WHEN event_date ~ '^[0-9]{4}/[0-9]{1,2}/[0-9]{1,2}$' THEN to_date(event_date, 'YYYY/MM/DD')
                WHEN event_date ~ '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{4}$' THEN to_date(event_date, 'DD/MM/YYYY')
                WHEN event_date ~ '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{2}$' THEN 
                    CASE 
                        WHEN event_date ~ '^[0-9]{1}/[0-9]{1,2}/[0-9]{2}$' THEN to_date(event_date, 'D/MM/YY')
                        ELSE to_date(event_date, 'D/M/YY')
                    END
                ELSE '1900-01-01'::date  -- fallback date for invalid formats
             END",
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    // Tests for DD/MM/YYYY format
    #[test]
    fn test_parse_dd_mm_yyyy() {
        let date = DateParser::parse_flexible_date("25/12/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
    }

    #[test]
    fn test_parse_dd_mm_yyyy_single_digit_day() {
        let date = DateParser::parse_flexible_date("05/12/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 5).unwrap());
    }

    #[test]
    fn test_parse_dd_mm_yyyy_single_digit_month() {
        let date = DateParser::parse_flexible_date("25/03/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 3, 25).unwrap());
    }

    #[test]
    fn test_parse_dd_mm_yyyy_leap_year() {
        let date = DateParser::parse_flexible_date("29/02/2024").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    // Tests for YYYY/MM/DD format
    #[test]
    fn test_parse_yyyy_mm_dd() {
        let date = DateParser::parse_flexible_date("2023/12/25").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
    }

    #[test]
    fn test_parse_yyyy_mm_dd_single_digits() {
        let date = DateParser::parse_flexible_date("2023/3/5").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 3, 5).unwrap());
    }

    #[test]
    fn test_parse_yyyy_mm_dd_new_year() {
        let date = DateParser::parse_flexible_date("2024/01/01").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    }

    // Tests for D/M/YY format with 2-digit year conversion
    #[test]
    fn test_parse_d_m_yy() {
        let date = DateParser::parse_flexible_date("5/3/23").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 3, 5).unwrap());
    }

    #[test]
    fn test_parse_d_m_yy_boundary_49() {
        // Year 49 should become 2049
        let date = DateParser::parse_flexible_date("1/1/49").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2049, 1, 1).unwrap());
    }

    #[test]
    fn test_parse_d_m_yy_boundary_50() {
        // Year 50 should become 1950
        let date = DateParser::parse_flexible_date("1/1/50").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1950, 1, 1).unwrap());
    }

    #[test]
    fn test_parse_d_m_yy_boundary_99() {
        // Year 99 should become 1999
        let date = DateParser::parse_flexible_date("31/12/99").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1999, 12, 31).unwrap());
    }

    #[test]
    fn test_parse_d_m_yy_boundary_00() {
        // Year 00 should become 2000
        let date = DateParser::parse_flexible_date("1/1/00").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
    }

    #[test]
    fn test_parse_d_m_yy_current_century() {
        // Years in the 20s should be interpreted as 2020s
        let date = DateParser::parse_flexible_date("15/6/25").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
    }

    // Tests for D/M/YYYY format
    #[test]
    fn test_parse_d_m_yyyy() {
        let date = DateParser::parse_flexible_date("5/3/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 3, 5).unwrap());
    }

    #[test]
    fn test_parse_d_m_yyyy_single_digit_both() {
        let date = DateParser::parse_flexible_date("1/1/2024").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    }

    #[test]
    fn test_parse_d_m_yyyy_different_centuries() {
        let date = DateParser::parse_flexible_date("10/5/1985").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1985, 5, 10).unwrap());
    }

    // Tests for DD/MM/YY format
    #[test]
    fn test_parse_dd_mm_yy() {
        let date = DateParser::parse_flexible_date("25/12/23").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
    }

    #[test]
    fn test_parse_dd_mm_yy_80s() {
        let date = DateParser::parse_flexible_date("15/08/85").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1985, 8, 15).unwrap());
    }

    // Edge cases and error conditions
    #[test]
    fn test_parse_invalid_format_no_slashes() {
        let result = DateParser::parse_flexible_date("25122023");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_wrong_separator() {
        let result = DateParser::parse_flexible_date("25-12-2023");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_too_many_parts() {
        let result = DateParser::parse_flexible_date("25/12/2023/extra");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_too_few_parts() {
        let result = DateParser::parse_flexible_date("25/12");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_date_values() {
        // Invalid day
        let result = DateParser::parse_flexible_date("32/12/2023");
        assert!(result.is_err());

        // Invalid month
        let result = DateParser::parse_flexible_date("25/13/2023");
        assert!(result.is_err());

        // Invalid leap year date
        let result = DateParser::parse_flexible_date("29/02/2023");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_string() {
        let result = DateParser::parse_flexible_date("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_string() {
        let result = DateParser::parse_flexible_date("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_non_numeric_parts() {
        let result = DateParser::parse_flexible_date("abc/def/ghi");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mixed_valid_invalid() {
        let result = DateParser::parse_flexible_date("25/abc/2023");
        assert!(result.is_err());
    }

    // Tests for ambiguous dates (where format matters)
    #[test]
    fn test_parse_ambiguous_date_dd_mm_yyyy() {
        // 05/03/2023 could be March 5th or May 3rd
        // Our parser should interpret this as DD/MM/YYYY (March 5th)
        let date = DateParser::parse_flexible_date("05/03/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 3, 5).unwrap());
    }

    #[test]
    fn test_parse_unambiguous_date_yyyy_mm_dd() {
        // 2023/05/03 is unambiguous as YYYY/MM/DD (May 3rd)
        let date = DateParser::parse_flexible_date("2023/05/03").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 5, 3).unwrap());
    }

    // Boundary testing for months and days
    #[test]
    fn test_parse_boundary_dates() {
        // First day of year
        let date = DateParser::parse_flexible_date("01/01/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());

        // Last day of year
        let date = DateParser::parse_flexible_date("31/12/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());

        // Last day of February (non-leap year)
        let date = DateParser::parse_flexible_date("28/02/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    // Tests for different year ranges
    #[test]
    fn test_parse_different_year_ranges() {
        // Far past
        let date = DateParser::parse_flexible_date("15/07/1900").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1900, 7, 15).unwrap());

        // Recent past
        let date = DateParser::parse_flexible_date("10/03/2020").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2020, 3, 10).unwrap());

        // Far future (within reasonable bounds)
        let date = DateParser::parse_flexible_date("05/11/2050").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2050, 11, 5).unwrap());
    }

    // Tests for format detection logic
    #[test]
    fn test_format_detection_priority() {
        // Test that YYYY/MM/DD is detected first when year is 4 digits in first position
        let date = DateParser::parse_flexible_date("2023/01/15").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 1, 15).unwrap());

        // Test that DD/MM/YYYY is detected when year is 4 digits in last position
        let date = DateParser::parse_flexible_date("15/01/2023").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 1, 15).unwrap());
    }

    // Comprehensive test with various realistic database values
    #[test]
    fn test_realistic_database_values() {
        let test_cases = vec![
            ("1/1/23", NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            ("31/12/99", NaiveDate::from_ymd_opt(1999, 12, 31).unwrap()),
            ("15/06/2024", NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            ("2022/03/10", NaiveDate::from_ymd_opt(2022, 3, 10).unwrap()),
            ("5/11/85", NaiveDate::from_ymd_opt(1985, 11, 5).unwrap()),
            ("29/02/2020", NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()), // Leap year
        ];

        for (input, expected) in test_cases {
            let result = DateParser::parse_flexible_date(input).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    // Test that demonstrates the 2-digit year conversion logic clearly
    #[test]
    fn test_two_digit_year_conversion_comprehensive() {
        // Test the boundary cases comprehensively
        let boundary_cases = vec![
            // Years 00-49 should become 2000-2049
            ("1/1/00", 2000),
            ("1/1/01", 2001),
            ("1/1/25", 2025), // Current year context
            ("1/1/49", 2049),
            // Years 50-99 should become 1950-1999
            ("1/1/50", 1950),
            ("1/1/75", 1975),
            ("1/1/85", 1985),
            ("1/1/99", 1999),
        ];

        for (input, expected_year) in boundary_cases {
            let result = DateParser::parse_flexible_date(input).unwrap();
            assert_eq!(result.year(), expected_year, "Failed for input: {} (expected year {})", input, expected_year);
        }
    }
}

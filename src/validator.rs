use crate::models::PhoneInfo;
use phonenumber::country::Id;
use phonenumber::Mode;
use std::str::FromStr;

pub fn validate_and_parse(number: &str, region: &str) -> Option<PhoneInfo> {
    let country_id = if !region.trim().is_empty() {
        Id::from_str(&region.to_uppercase()).ok()
    } else {
        None
    };

    let parsed_phone = phonenumber::parse(country_id, number)
        .or_else(|_| phonenumber::parse(None, number))
        .ok()?;

    if !phonenumber::is_valid(&parsed_phone) {
        return None;
    }

    let international = parsed_phone.format().mode(Mode::International).to_string();
    let national = parsed_phone.format().mode(Mode::National).to_string();
    let e164 = parsed_phone.format().mode(Mode::E164).to_string();

    let country = parsed_phone
        .country()
        .id()
        .map(|id| id.as_ref().to_string())
        .unwrap_or_else(|| region.to_uppercase());

    Some(PhoneInfo {
        international,
        national,
        e164,
        country,
        carrier: "Mobile / Fixed".to_string(),
        timezone: vec!["UTC".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_phone_parsing_us() {
        let res = validate_and_parse("+16502530000", "US");
        assert!(res.is_some());
        let info = res.unwrap();
        assert_eq!(info.e164, "+16502530000");
        assert_eq!(info.country, "US");
    }

    #[test]
    fn test_valid_phone_parsing_pe() {
        let res = validate_and_parse("+51987654321", "PE");
        assert!(res.is_some());
        let info = res.unwrap();
        assert_eq!(info.country, "PE");
    }

    #[test]
    fn test_valid_phone_parsing_es() {
        let res = validate_and_parse("+34912345678", "ES");
        assert!(res.is_some());
        let info = res.unwrap();
        assert_eq!(info.country, "ES");
    }

    #[test]
    fn test_invalid_phone_parsing() {
        let res = validate_and_parse("12345", "US");
        assert!(res.is_none());
    }
}

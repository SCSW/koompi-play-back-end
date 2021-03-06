extern crate jsonwebtoken;
use jsonwebtoken::Header;
// use jsonwebtoken::{EncodingKey, DecodingKey};
use jsonwebtoken::Algorithm;
use jsonwebtoken::Validation;
use jsonwebtoken::TokenData;

extern crate chrono;
use chrono::DateTime;
use chrono::Utc;

use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,        // Optional. Audience
    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>, // Required (validate_exp defaults to true in validation). Expiration time
    // #[serde(with = "jwt_numeric_date")]
    // pub iat: DateTime<Utc>,  // Optional. Issued at
    // pub iss: String,       // Optional. Issuer
    // #[serde(with = "jwt_numeric_date")]
    // nbf: DateTime<Utc>,  // Optional. Not Before
    // pub sub: String,        // Optional. Subject (whom token refers to)
    // pub user_name: String,
    // pub user_password: String,
    pub user_email: String,
    pub user_role: String
}

mod jwt_numeric_date {
    //! Custom serialization of DateTime<Utc> to conform with the JWT spec (RFC 7519 section 2, "Numeric Date")
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single() // If there are multiple or no valid DateTimes from timestamp, return None
            .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"))
    }

    #[cfg(test)]
    mod tests {
        const EXPECTED_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJDdXN0b20gRGF0ZVRpbWUgc2VyL2RlIiwiaWF0IjowLCJleHAiOjMyNTAzNjgwMDAwfQ.RTgha0S53MjPC2pMA4e2oMzaBxSY3DMjiYR2qFfV55A";

        use super::super::{Claims, SECRET};

        #[test]
        fn round_trip() {
            let sub = "Custom DateTime ser/de".to_string();
            let iat = Utc.timestamp(0, 0);
            let exp = Utc.timestamp(32503680000, 0);

            let claims = Claims { sub: sub.clone(), iat, exp };

            let token =
                encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET.as_ref()))
                    .expect("Failed to encode claims");

            assert_eq!(&token, EXPECTED_TOKEN);

            let decoded = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(SECRET.as_ref()),
                &Validation::default(),
            )
            .expect("Failed to decode token");

            assert_eq!(decoded.claims, claims);
        }

        #[test]
        fn should_fail_on_invalid_timestamp() {
            // A token with the expiry of i64::MAX + 1
            let overflow_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJDdXN0b20gRGF0ZVRpbWUgc2VyL2RlIiwiaWF0IjowLCJleHAiOjkyMjMzNzIwMzY4NTQ3NzYwMDB9.G2PKreA27U8_xOwuIeCYXacFYeR46f9FyENIZfCrvEc";

            let decode_result =
                decode::<Claims>(&overflow_token, SECRET.as_ref(), &Validation::default());

            assert!(decode_result.is_err());
        }
    }
}

use chrono::Duration;
pub fn generate_token(login_email: String, login_role: String) -> String {
    let issue_time: DateTime<Utc> = Utc::now();
    //declare 1day durations
    let duration = Duration::days(1);
    // let duration = Duration::seconds(10i64);
    // let duration = Duration::minutes(5i64);
    let expire_time = issue_time.checked_add_signed(duration).unwrap();

    let claims = Claims {
        aud:            String::from("koompiPlay"),
        exp:            expire_time,
        // iat:            issue_time,
        // iss:            String::from("koompiPlay"),
        // sub:            String::from("login"),
        // user_name:      login_name,
        // user_password:  login_password,
        user_email:     login_email,
        user_role:      login_role,
    };

    let token = jsonwebtoken::encode(&Header::default(), &claims, "secret".as_ref()).unwrap();
    return token;   
}

pub fn decode_token(token: String) -> jsonwebtoken::TokenData<Claims> {
    // println!("in decode token: {}", token);
    jsonwebtoken::decode::<Claims>(&token, "secret".as_ref(), &Validation::default()).unwrap()
}

use serde_json::value::Value;
use std::collections::HashSet;
use jsonwebtoken::errors::Error;
pub fn decode_token_result(token: String) -> Result<jsonwebtoken::TokenData<Claims>, Error> {
    // println!("in decode token: {}", token);

    // let mut aud_hashset = HashSet::new();
    // aud_hashset.insert("koompiPlay".to_string());
    
    // let aud_hashset = json!("koompiPlay");
    let aud_value = Some(serde_json::value::Value::String(String::from("koompiPlay")));

    let mut validation = Validation {
        // aud: Some(aud_hashset),
        // aud: Some(String::from("koompiPlay")),
        aud: aud_value,
        ..Default::default()
    };
    // jsonwebtoken::decode::<Claims>(&token, "secret".as_ref(), &Validation::default())
    jsonwebtoken::decode::<Claims>(&token, "secret".as_ref(), &validation)
}
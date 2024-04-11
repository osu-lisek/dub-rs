use serde::Deserialize;
use tracing::error;

macro_rules! country_enum {
    ($($country:ident = $value:expr),*) => {
        #[repr(u8)]
        #[derive(Debug, Clone)]
        pub enum Country {
            $($country = $value),*
        }

        impl Country {
            pub fn from_code(code: &str) -> Option<u8> {
                match code {
                    $(stringify!($country) => Some(Country::$country as u8)),*,
                    _ => None,
                }
            }

            pub fn to_byte(&self) -> u8 {
                self.clone() as u8
            }
        }
    };
}

country_enum!(
    XX = 0,
    AD = 3,
    AF = 5,
    AI = 7,
    AL = 8,
    AM = 9,
    AO = 11,
    AQ = 12,
    AR = 13,
    AS = 14,
    AT = 15,
    AU = 16,
    AW = 17,
    AZ = 18,
    BB = 20,
    BD = 21,
    BE = 22,
    BF = 23,
    BG = 24,
    BH = 25,
    BI = 26,
    BJ = 27,
    BM = 28,
    BO = 30,
    BR = 31,
    BS = 32,
    BT = 33,
    BV = 34,
    BW = 35,
    BY = 36,
    BZ = 37,
    CA = 38,
    CC = 39,
    CF = 41,
    CH = 43,
    CK = 45,
    CL = 46,
    CM = 47,
    CN = 48,
    CO = 49,
    CR = 50,
    CU = 51,
    CV = 52,
    CX = 53,
    CY = 54,
    CZ = 55,
    DE = 56,
    DJ = 57,
    DK = 58,
    DM = 59,
    DO = 60,
    DZ = 61,
    EC = 62,
    EE = 63,
    EG = 64,
    EH = 65,
    ER = 66,
    ES = 67,
    ET = 68,
    FI = 69,
    FJ = 70,
    FK = 71,
    FO = 73,
    FR = 74,
    GA = 76,
    GB = 77,
    GD = 78,
    GE = 79,
    GF = 80,
    GH = 81,
    GI = 82,
    GL = 83,
    GM = 84,
    GN = 85,
    GP = 86,
    GQ = 87,
    GR = 88,
    GT = 90,
    GU = 91,
    GW = 92,
    GY = 93,
    HK = 94,
    HN = 96,
    HR = 97,
    HT = 98,
    HU = 99,
    ID = 100,
    IE = 101,
    IL = 102,
    IN = 103,
    IO = 104,
    IQ = 105,
    IS = 107,
    IT = 108,
    JM = 109,
    JO = 110,
    JP = 111,
    KE = 112,
    KG = 113,
    KH = 114,
    KI = 115,
    KM = 116,
    KW = 120,
    KY = 121,
    KZ = 122,
    LB = 124,
    LI = 126,
    LK = 127,
    LR = 128,
    LS = 129,
    LT = 130,
    LU = 131,
    LV = 132,
    MA = 134,
    MC = 135,
    MG = 137,
    MH = 138,
    ML = 140,
    MM = 141,
    MN = 142,
    MP = 144,
    MQ = 145,
    MR = 146,
    MS = 147,
    MT = 148,
    MU = 149,
    MV = 150,
    MW = 151,
    MX = 152,
    MY = 153,
    MZ = 154,
    NA = 155,
    NC = 156,
    NE = 157,
    NF = 158,
    NG = 159,
    NI = 160,
    NL = 161,
    NO = 162,
    NP = 163,
    NR = 164,
    NU = 165,
    NZ = 166,
    OM = 167,
    PA = 168,
    PE = 169,
    PF = 170,
    PG = 171,
    PH = 172,
    PK = 173,
    PL = 174,
    PN = 176,
    PR = 177,
    PS = 178,
    PT = 179,
    PW = 180,
    PY = 181,
    QA = 182,
    RE = 183,
    RO = 184,
    RU = 185,
    RW = 186,
    SA = 187,
    SB = 188,
    SC = 189,
    SD = 190,
    SE = 191,
    SG = 192,
    SI = 194,
    SJ = 195,
    SK = 196,
    SL = 197,
    SM = 198,
    SN = 199,
    SO = 200,
    SR = 201,
    ST = 202,
    SV = 203,
    SZ = 205,
    TC = 206,
    TD = 207,
    TF = 208,
    TG = 209,
    TH = 210,
    TJ = 211,
    TK = 212,
    TM = 213,
    TN = 214,
    TO = 215,
    TR = 217,
    TT = 218,
    TV = 219,
    TW = 220,
    TZ = 221,
    UA = 222,
    UG = 223,
    US = 225,
    UY = 226,
    UZ = 227,
    VE = 230,
    VN = 233,
    VU = 234,
    WF = 235,
    WS = 236,
    YE = 237,
    YT = 238,
    RS = 239,
    ZA = 240,
    ZM = 241,
    ME = 242,
    ZW = 243,
    AX = 247,
    GG = 248,
    IM = 249,
    JE = 250,
    MF = 252
);

#[derive(Debug, Deserialize, Clone)]
pub struct IpApiResponse {
    #[serde(rename = "countryCode")]
    pub code: String,
    pub lat: f32,
    pub lon: f32,
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn get_ip_info(ip: Option<String>) -> Option<IpApiResponse> {
    #[cfg(debug_assertions)]
    return Some(IpApiResponse {
        code: "DE".to_string(),
        lat: 0.0,
        lon: 0.0,
    });

    let resp = reqwest::get(format!(
        "http://ip-api.com/json/{}?fields=countryCode,lat,lon",
        ip.clone().unwrap_or("".to_string())
    ))
    .await
    .unwrap()
    .json::<IpApiResponse>()
    .await;

    if let Err(_error) = resp {
        error!("Error looking up for ip: {:#?}", ip.unwrap_or_default());
        return None;
    }

    return Some(resp.unwrap());
}

pub fn _code_from_string(country_code: String) -> u8 {
    let country_code = country_code.to_uppercase();

    if let Some(country) = Country::from_code(&country_code) {
        return country;
    }

    0
}

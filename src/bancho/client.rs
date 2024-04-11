#[derive(Debug, Clone)]
pub struct HWID {
    pub plain: String,
    pub mac: String,
    pub uid: String,
    pub disk: String,
}

#[derive(Debug, Clone)]
pub struct ClientData {
    pub client_version: String,
    pub time_offset: i8,
    pub hwid: HWID,
}

impl Default for ClientData {
    fn default() -> Self {
        Self {
            client_version: String::new(),
            time_offset: 0,
            hwid: HWID::default(),
        }
    }
}

impl Default for HWID {
    fn default() -> Self {
        Self {
            plain: String::new(),
            mac: String::new(),
            uid: String::new(),
            disk: String::new(),
        }
    }
}

impl ClientData {
    pub fn from(data: String) -> Self {
        let mut splitted_data = data.split("|");
        let client_version = splitted_data.next();
        let time_offset = splitted_data.next();
        let _ = splitted_data.next();

        let unparsed_hiwd = splitted_data.next();
        let hwid = HWID::from(unparsed_hiwd.unwrap().to_string());

        Self {
            client_version: client_version.unwrap().to_string(),
            time_offset: time_offset.unwrap().parse::<i8>().unwrap(),
            hwid,
        }
    }
}

impl HWID {
    pub fn from(data: String) -> Self {
        let mut splitted_data = data.split(":");
        let plain = splitted_data.next();
        let mac = splitted_data.next();
        let uid = splitted_data.next();
        let disk = splitted_data.next();

        Self {
            plain: plain.unwrap().to_string(),
            mac: mac.unwrap().to_string(),
            uid: uid.unwrap().to_string(),
            disk: disk.unwrap().to_string(),
        }
    }
}

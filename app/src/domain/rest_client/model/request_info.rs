#[derive(Clone, Debug, PartialEq)]
pub enum RequestCommand {
    None,
    Run,
    CopyCUrl,
    CopyCUrlWin,
}

#[derive(Clone, Debug)]
pub struct RequestInfo {
    pub id: i32,
    pub project_id: i32,
    pub url: String,
    pub name: String,
    pub method: String,
    pub command: RequestCommand,
}

impl RequestInfo {
    pub fn new(id: i32, project_id: i32, url: String, name: String, method: String) -> Self {
        Self { id, project_id, url, method, name, command: RequestCommand::None }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            project_id: 0,
            url: "".to_owned(),
            method: "".to_owned(),
            name: "".to_owned(),
            command: RequestCommand::None,
        }
    }

    pub fn display_name(&self) -> String {
        if !self.name.is_empty() { self.name.to_owned() } else { self.url.to_owned() }
    }
}

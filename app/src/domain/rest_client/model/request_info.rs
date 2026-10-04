use getset::{CopyGetters, Getters, Setters};

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum RequestCommand {
    None,
    Run,
    CopyCUrl,
    CopyCUrlWin,
}

#[derive(Clone, Debug, Getters, CopyGetters, Setters)]
pub struct RequestInfo {
    #[get_copy="pub"]
    id: i32,
    #[get_copy="pub"]
    project_id: i32,
    #[getset(get="pub", set="pub")]
    url: String,
    #[getset(get="pub", set="pub")]
    name: String,
    #[getset(get="pub", set="pub")]
    method: String,
    #[getset(get_copy="pub", set="pub")]
    command: RequestCommand,
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

    pub fn id_str(&self) -> String {
        self.id.to_string()
    }
}

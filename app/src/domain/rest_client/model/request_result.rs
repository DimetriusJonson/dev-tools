use leptos::prelude::{ReadSignal, RwSignal, Set, WriteSignal};

pub type RequestResultAttachment = (String, String);
pub type RequestResultHeaders = Vec<(String, String)>;

#[derive(Clone, Default)]
pub struct RequestResult {
    status_code: RwSignal<String>,
    size: RwSignal<Option<u64>>,
    request_time: RwSignal<u64>,
    body: RwSignal<String>,
    attachment: RwSignal<RequestResultAttachment>,
    image: RwSignal<String>,
    lang: RwSignal<String>,
    headers: RwSignal<RequestResultHeaders>,
    request_raw: RwSignal<String>,
}

impl RequestResult {
    #[inline]
    pub fn status_code(&self) -> ReadSignal<String> {
        self.status_code.read_only()
    }

    #[inline]
    pub fn set_status_code(&self) -> WriteSignal<String> {
        self.status_code.write_only()
    }

    #[inline]
    pub fn size(&self) -> ReadSignal<Option<u64>> {
        self.size.read_only()
    }

    #[inline]
    pub fn set_size(&self) -> WriteSignal<Option<u64>> {
        self.size.write_only()
    }

    #[inline]
    pub fn request_time(&self) -> ReadSignal<u64> {
        self.request_time.read_only()
    }

    #[inline]
    pub fn set_request_time(&self) -> WriteSignal<u64> {
        self.request_time.write_only()
    }

    #[inline]
    pub fn body(&self) -> ReadSignal<String> {
        self.body.read_only()
    }

    #[inline]
    pub fn set_body(&self) -> WriteSignal<String> {
        self.body.write_only()
    }

    #[inline]
    pub fn attachment(&self) -> ReadSignal<RequestResultAttachment> {
        self.attachment.read_only()
    }

    #[inline]
    pub fn set_attachment(&self) -> WriteSignal<RequestResultAttachment> {
        self.attachment.write_only()
    }

    #[inline]
    pub fn image(&self) -> ReadSignal<String> {
        self.image.read_only()
    }

    #[inline]
    pub fn set_image(&self) -> WriteSignal<String> {
        self.image.write_only()
    }

    #[inline]
    pub fn lang(&self) -> ReadSignal<String> {
        self.lang.read_only()
    }

    #[inline]
    pub fn set_lang(&self) -> WriteSignal<String> {
        self.lang.write_only()
    }

    #[inline]
    pub fn headers(&self) -> ReadSignal<RequestResultHeaders> {
        self.headers.read_only()
    }

    #[inline]
    pub fn set_headers(&self) -> WriteSignal<RequestResultHeaders> {
        self.headers.write_only()
    }

    #[inline]
    pub fn request_raw(&self) -> ReadSignal<String> {
        self.request_raw.read_only()
    }

    #[inline]
    pub fn set_request_raw(&self) -> WriteSignal<String> {
        self.request_raw.write_only()
    }

    pub fn clear(&self) {
        self.status_code.set("".to_owned());
        self.size.set(None);
        self.request_time.set(0);
        self.body.set("".to_owned());
        self.lang.set("".to_owned());
        self.headers.set(Vec::new());
        self.request_raw.set("".to_owned());
        self.attachment.set(("".to_owned(), "".to_owned()));
        self.image.set("".to_owned());
    }
}

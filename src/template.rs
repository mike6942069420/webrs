use sailfish::RenderError;
use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "index.html")]
pub struct Template<'a> {
    pub nbusers: usize,
    pub nonce: &'a str,
    pub messages: Vec<String>,
}

#[inline(always)]
pub fn render(template: Template) -> Result<String, RenderError> {
    template.render_once()
}

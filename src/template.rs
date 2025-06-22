use sailfish::RenderError;
use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "index.html")]
struct Template {
    nbusers: u32,
}

pub fn render(nbusers: u32) -> Result<String, RenderError> {
    let template = Template { nbusers };
    template.render_once()
}

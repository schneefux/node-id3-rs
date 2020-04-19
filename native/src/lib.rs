use neon::prelude::*;
use id3::{Tag, Content, Frame, Version};

// hack for direct access to Tag.frames
struct ExposedTag {
    pub frames: Vec<Frame>,
}

fn replace_tag_at_index_sync(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let path = cx.argument::<JsString>(0)?.value();
    let index = cx.argument::<JsNumber>(1)?.value() as usize;
    let id = cx.argument::<JsString>(2)?.value();
    let mut data = cx.argument::<JsString>(3)?.value();

    let binary_data = base64::decode(data).unwrap();
    let content = Content::Unknown(binary_data);
    let frame = Frame::with_content(id, content);

    let mut tag = Tag::read_from_path(path).unwrap();
    let mut tag_exposed: ExposedTag = unsafe {
        std::mem::transmute(tag)
    };

    tag_exposed.frames.remove(index);
    tag_exposed.frames.insert(index, frame);

    tag = unsafe {
        std::mem::transmute(tag_exposed)
    };
    tag.write_to_path(cx.argument::<JsString>(0)?.value(), Version::Id3v23).unwrap();

    Ok(cx.boolean(true))
}

fn read_tags_sync(mut cx: FunctionContext) -> JsResult<JsArray> {
    let path = cx.argument::<JsString>(0)?.value();

    let tag = Tag::read_from_path(path).unwrap();
    let array = JsArray::new(&mut cx, tag.frames().count() as u32);

    for (index, frame) in tag.frames().enumerate() {
        let content = frame.content();
        let js_id = cx.string(frame.id());
        let obj = JsObject::new(&mut cx);
        obj.set(&mut cx, "id", js_id);
        match content {
            Content::Text(text) => {
                let js_text = cx.string(text);
                obj.set(&mut cx, "text", js_text);
            },
            Content::ExtendedText(extended) => {
                let js_description = cx.string(extended.clone().description);
                let js_value = cx.string(extended.clone().value);
                obj.set(&mut cx, "description", js_description);
                obj.set(&mut cx, "value", js_value);
            },
            Content::Link(link) => {
                let js_link = cx.string(link);
                obj.set(&mut cx, "link", js_link);
            },
            Content::ExtendedLink(extended) => {
                let js_description = cx.string(extended.clone().description);
                let js_link = cx.string(extended.clone().link);
                obj.set(&mut cx, "description", js_description);
                obj.set(&mut cx, "link", js_link);
            },
            Content::Comment(comment) => {
                let js_lang = cx.string(comment.clone().lang);
                let js_description = cx.string(comment.clone().description);
                let js_text = cx.string(comment.clone().text);
                obj.set(&mut cx, "lang", js_lang);
                obj.set(&mut cx, "description", js_description);
                obj.set(&mut cx, "text", js_text);
            },
            Content::Lyrics(lyrics) => {
                let js_lang = cx.string(lyrics.clone().lang);
                let js_description = cx.string(lyrics.clone().description);
                let js_text = cx.string(lyrics.clone().text);
                obj.set(&mut cx, "lang", js_lang);
                obj.set(&mut cx, "description", js_description);
                obj.set(&mut cx, "text", js_text);
            },
            Content::SynchronisedLyrics(lyrics) => {
                let js_lang = cx.string(lyrics.clone().lang);
                obj.set(&mut cx, "lang", js_lang);
                /* TODO */
            },
            Content::Picture(pic) => {
                let js_mimetype = cx.string(pic.clone().mime_type);
                let js_description = cx.string(pic.clone().description);
                let js_data = cx.string(base64::encode(pic.clone().data));
                obj.set(&mut cx, "mime_type", js_mimetype);
                obj.set(&mut cx, "description", js_description);
                obj.set(&mut cx, "data", js_data);
                /* TODO picture type */
            },
            Content::Unknown(vec) => {
                let js_data = cx.string(base64::encode(vec));
                obj.set(&mut cx, "data", js_data);
            },
        };

        array.set(&mut cx, index as u32, obj).unwrap();
    }

    Ok(array)
}

register_module!(mut cx, {
    cx.export_function("readTagsSync", read_tags_sync);
    cx.export_function("replaceTagAtIndexSync", replace_tag_at_index_sync)
});

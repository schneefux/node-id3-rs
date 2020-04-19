use neon::prelude::*;
use id3::{Tag, Content, Frame, Version};

struct ReadTagsTask {
    path: String,
}

impl Task for ReadTagsTask {
    type Output = Tag;
    type Error = String;
    type JsEvent = JsArray;

    fn perform(&self) -> Result<Tag, String> {
        let tag = Tag::read_from_path(self.path.clone()).unwrap();

        Ok(tag)
    }

    fn complete(self, cx: TaskContext, result: Result<Tag, String>) -> JsResult<JsArray> {
        let array = tag_to_array(cx, result.unwrap());

        Ok(array)
    }
}

struct ReplaceTagTask {
    path: String,
    index: usize,
    id: String,
    data: String,
}

impl Task for ReplaceTagTask {
    type Output = String;
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<String, String> {
        replace_tag(self.path.clone(), self.index, self.id.clone(), self.data.clone());

        Ok("".to_string())
    }

    fn complete(self, mut cx: TaskContext, _result: Result<String, String>) -> JsResult<JsUndefined> {
        Ok(cx.undefined())
    }
}

/**
 * Turn a tag into a JS array of objects.
 */
fn tag_to_array<'a, C: Context<'a>>(mut cx: C, tag: Tag) -> Handle<'a, JsArray> {
    let array = JsArray::new(&mut cx, tag.frames().count() as u32);

    for (index, frame) in tag.frames().enumerate() {
        let content = frame.content();
        let js_id = cx.string(frame.id());
        let obj = JsObject::new(&mut cx);
        obj.set(&mut cx, "id", js_id).unwrap();
        match content {
            Content::Text(text) => {
                let js_text = cx.string(text);
                obj.set(&mut cx, "text", js_text).unwrap();
            },
            Content::ExtendedText(extended) => {
                let js_description = cx.string(extended.clone().description);
                let js_value = cx.string(extended.clone().value);
                obj.set(&mut cx, "description", js_description).unwrap();
                obj.set(&mut cx, "value", js_value).unwrap();
            },
            Content::Link(link) => {
                let js_link = cx.string(link);
                obj.set(&mut cx, "link", js_link).unwrap();
            },
            Content::ExtendedLink(extended) => {
                let js_description = cx.string(extended.clone().description);
                let js_link = cx.string(extended.clone().link);
                obj.set(&mut cx, "description", js_description).unwrap();
                obj.set(&mut cx, "link", js_link).unwrap();
            },
            Content::Comment(comment) => {
                let js_lang = cx.string(comment.clone().lang);
                let js_description = cx.string(comment.clone().description);
                let js_text = cx.string(comment.clone().text);
                obj.set(&mut cx, "lang", js_lang).unwrap();
                obj.set(&mut cx, "description", js_description).unwrap();
                obj.set(&mut cx, "text", js_text).unwrap();
            },
            Content::Lyrics(lyrics) => {
                let js_lang = cx.string(lyrics.clone().lang);
                let js_description = cx.string(lyrics.clone().description);
                let js_text = cx.string(lyrics.clone().text);
                obj.set(&mut cx, "lang", js_lang).unwrap();
                obj.set(&mut cx, "description", js_description).unwrap();
                obj.set(&mut cx, "text", js_text).unwrap();
            },
            Content::SynchronisedLyrics(lyrics) => {
                let js_lang = cx.string(lyrics.clone().lang);
                obj.set(&mut cx, "lang", js_lang).unwrap();
                /* TODO */
            },
            Content::Picture(pic) => {
                let js_mimetype = cx.string(pic.clone().mime_type);
                let js_description = cx.string(pic.clone().description);
                let js_data = cx.string(base64::encode(pic.clone().data));
                obj.set(&mut cx, "mime_type", js_mimetype).unwrap();
                obj.set(&mut cx, "description", js_description).unwrap();
                obj.set(&mut cx, "data", js_data).unwrap();
                /* TODO picture type */
            },
            Content::Unknown(vec) => {
                let js_data = cx.string(base64::encode(vec));
                obj.set(&mut cx, "data", js_data).unwrap();
            },
        };

        array.set(&mut cx, index as u32, obj).unwrap();
    }

    array
}

/**
 * Read an MP3 and replace the tag frame at the given position.
 * If data is valid base64, a binary frame is written, else, a text frame.
 * If the index is out of bounds, the frame is appended.
 */
fn replace_tag(path: String, index: usize, id: String, data: String) {
    let content = base64::decode(data.clone())
        .map(|binary| Content::Unknown(binary))
        .unwrap_or(Content::Text(data));
    let frame = Frame::with_content(id, content);

    let mut tag = Tag::read_from_path(path.clone()).unwrap();
    let mut tag_exposed: ExposedTag = unsafe {
        std::mem::transmute(tag)
    };

    if index > 0 && index < tag_exposed.frames.len() {
      tag_exposed.frames.remove(index);
      tag_exposed.frames.insert(index, frame);
    } else {
      tag_exposed.frames.push(frame);
    }

    tag = unsafe {
        std::mem::transmute(tag_exposed)
    };
    tag.write_to_path(path, Version::Id3v24).unwrap(); // TODO detect version
}

// hack for direct access to Tag.frames
struct ExposedTag {
    pub frames: Vec<Frame>,
}

/**
 * Async javascript function `replaceTagAtIndex`.
 */
fn replace_tag_at_index(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let index = cx.argument::<JsNumber>(1)?.value() as usize;
    let id = cx.argument::<JsString>(2)?.value();
    let data = cx.argument::<JsString>(3)?.value();
    let f = cx.argument::<JsFunction>(4)?;

    let task = ReplaceTagTask {
        path,
        index,
        id,
        data,
    };
    task.schedule(f);

    Ok(cx.undefined())
}

/**
 * Sync javascript function `replaceTagAtIndexSync`.
 */
fn replace_tag_at_index_sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let index = cx.argument::<JsNumber>(1)?.value() as usize;
    let id = cx.argument::<JsString>(2)?.value();
    let data = cx.argument::<JsString>(3)?.value();

    replace_tag(path, index, id, data);
    Ok(cx.undefined())
}

/**
 * Async javascript function `readTags`.
 */
fn read_tags(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let path = cx.argument::<JsString>(0)?.value();
    let f = cx.argument::<JsFunction>(1)?;

    let task = ReadTagsTask {
        path,
    };
    task.schedule(f);

    Ok(cx.undefined())
}

/**
 * Sync javascript function `readTagsSync`.
 */
fn read_tags_sync(mut cx: FunctionContext) -> JsResult<JsArray> {
    let path = cx.argument::<JsString>(0)?.value();

    let tag = Tag::read_from_path(path).unwrap();
    let array = tag_to_array(cx, tag);

    Ok(array)
}

register_module!(mut cx, {
    cx.export_function("readTags", read_tags).unwrap();
    cx.export_function("readTagsSync", read_tags_sync).unwrap();
    cx.export_function("replaceTagAtIndex", replace_tag_at_index).unwrap();
    cx.export_function("replaceTagAtIndexSync", replace_tag_at_index_sync).unwrap();
    Ok(())
});

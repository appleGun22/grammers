// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use super::attributes::Attribute;
use crate::types::{Media, ReplyMarkup, Uploaded};
use grammers_tl_types as tl;
use web_time::{SystemTime, UNIX_EPOCH};

// https://github.com/telegramdesktop/tdesktop/blob/e7fbcce9d9f0a8944eb2c34e74bd01b8776cb891/Telegram/SourceFiles/data/data_scheduled_messages.h#L52
const SCHEDULE_ONCE_ONLINE: i32 = 0x7ffffffe;

/// Construct and send rich text messages with various options.
#[derive(Clone, Default)]
pub struct InputMessage {
    pub(crate) background: bool,
    pub(crate) clear_draft: bool,
    pub(crate) entities: Vec<tl::enums::MessageEntity>,
    pub(crate) invert_media: bool,
    pub(crate) link_preview: bool,
    pub(crate) reply_markup: Option<tl::enums::ReplyMarkup>,
    pub(crate) reply_to: Option<i32>,
    pub(crate) schedule_date: Option<i32>,
    pub(crate) silent: bool,
    pub(crate) text: String,
    pub(crate) media: Option<tl::enums::InputMedia>,
    media_ttl: Option<i32>,
    mime_type: Option<String>,
}

impl InputMessage {
    /// Whether to "send this message as a background message".
    ///
    /// This description is taken from <https://core.telegram.org/method/messages.sendMessage>.
    pub fn background(mut self, background: bool) -> Self {
        self.background = background;
        self
    }

    /// Whether the draft in this chat, if any, should be cleared.
    pub fn clear_draft(mut self, clear_draft: bool) -> Self {
        self.clear_draft = clear_draft;
        self
    }

    /// The formatting entities within the message (such as bold, italics, etc.).
    pub fn fmt_entities(mut self, entities: Vec<tl::enums::MessageEntity>) -> Self {
        self.entities = entities;
        self
    }

    /// Whether the media will be inverted.
    ///
    /// If inverted, photos, videos, and documents will appear at the bottom and link previews at the top of the message.
    pub fn invert_media(mut self, invert_media: bool) -> Self {
        self.invert_media = invert_media;
        self
    }

    /// Whether the link preview be shown for the message.
    ///
    /// This has no effect when sending media, which cannot contain a link preview.
    pub fn link_preview(mut self, link_preview: bool) -> Self {
        self.link_preview = link_preview;
        self
    }

    /// Defines the suggested reply markup for the message (such as adding inline buttons).
    /// This will be displayed below the message.
    ///
    /// Only bot accounts can make use of the reply markup feature (a user attempting to send a
    /// message with a reply markup will result in the markup being ignored by Telegram).
    ///
    /// The user is free to ignore the markup and continue sending usual text messages.
    ///
    /// See [`crate::reply_markup`] for the different available markups along with how
    /// they behave.
    pub fn reply_markup<RM: ReplyMarkup>(mut self, markup: &RM) -> Self {
        self.reply_markup = Some(markup.to_reply_markup().raw);
        self
    }

    /// The message identifier to which this message should reply to, if any.
    ///
    /// Otherwise, this message will not be a reply to any other.
    pub fn reply_to(mut self, reply_to: Option<i32>) -> Self {
        self.reply_to = reply_to;
        self
    }

    /// If set to a distant enough future time, the message won't be sent immediately,
    /// and instead it will be scheduled to be automatically sent at a later time.
    ///
    /// This scheduling is done server-side, and may not be accurate to the second.
    ///
    /// Bot accounts cannot schedule messages.
    pub fn schedule_date(mut self, schedule_date: Option<SystemTime>) -> Self {
        self.schedule_date = schedule_date.map(|t| {
            t.duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i32)
                .unwrap_or(0)
        });
        self
    }

    /// Schedule the message to be sent once the person comes online.
    ///
    /// This only works in private chats, and only if the person has their
    /// last seen visible.
    ///
    /// Bot accounts cannot schedule messages.
    pub fn schedule_once_online(mut self) -> Self {
        self.schedule_date = Some(SCHEDULE_ONCE_ONLINE);
        self
    }

    /// Whether the message should notify people or not.
    ///
    /// Defaults to `false`, which means it will notify them. Set it to `true`
    /// to alter this behaviour.
    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// Include the uploaded file as a photo in the message.
    ///
    /// The Telegram server will compress the image and convert it to JPEG format if necessary.
    ///
    /// The text will be the caption of the photo, which may be empty for no caption.
    pub fn photo(mut self, file: Uploaded) -> Self {
        self.media = Some(
            (tl::types::InputMediaUploadedPhoto {
                spoiler: false,
                file: file.raw,
                stickers: None,
                ttl_seconds: self.media_ttl,
            })
            .into(),
        );
        self
    }

    /// Include an external photo in the message.
    ///
    /// The Telegram server will download and compress the image and convert it to JPEG format if
    /// necessary.
    ///
    /// The text will be the caption of the photo, which may be empty for no caption.
    pub fn photo_url(mut self, url: impl Into<String>) -> Self {
        self.media = Some(
            (tl::types::InputMediaPhotoExternal {
                spoiler: false,
                url: url.into(),
                ttl_seconds: self.media_ttl,
            })
            .into(),
        );
        self
    }

    /// Include the uploaded file as a document in the message.
    ///
    /// You can use this to send videos, stickers, audios, or uncompressed photos.
    ///
    /// The text will be the caption of the document, which may be empty for no caption.
    pub fn document(mut self, file: Uploaded) -> Self {
        let mime_type = self.get_file_mime(&file);
        let file_name = file.name().to_string();
        self.media = Some(
            (tl::types::InputMediaUploadedDocument {
                nosound_video: false,
                force_file: false,
                spoiler: false,
                file: file.raw,
                thumb: None,
                mime_type,
                attributes: vec![(tl::types::DocumentAttributeFilename { file_name }).into()],
                stickers: None,
                ttl_seconds: self.media_ttl,
                video_cover: None,
                video_timestamp: None,
            })
            .into(),
        );
        self
    }

    /// Include a media in the message using the raw TL types.
    ///
    /// You can use this to send any media using the raw TL types that don't have
    /// a specific method in this builder such as Dice, Polls, etc.
    ///
    /// This can also be used to send media with a file reference, see `InputMediaDocument`
    /// and `InputMediaPhoto` in the `grammers-tl-types` crate.
    ///
    /// The text will be the caption of the media, which may be empty for no caption.
    pub fn media<M: Into<tl::enums::InputMedia>>(mut self, media: M) -> Self {
        self.media = Some(media.into());
        self
    }

    /// Include the video file with thumb in the message.
    ///
    /// The text will be the caption of the document, which may be empty for no caption.
    ///
    /// # Examples
    ///
    /// ```
    /// async fn f(client: &mut grammers_client::Client) -> Result<(), Box<dyn std::error::Error>> {
    ///     use grammers_client::{InputMessage};
    ///
    ///     let video = client.upload_file("video.mp4").await?;
    ///     let thumb = client.upload_file("thumb.png").await?;
    ///     let message = InputMessage::text("").document(video).thumbnail(thumb);
    ///     Ok(())
    /// }
    /// ```
    pub fn thumbnail(mut self, thumb: Uploaded) -> Self {
        if let Some(tl::enums::InputMedia::UploadedDocument(document)) = &mut self.media {
            document.thumb = Some(thumb.raw);
        }
        self
    }

    /// Include an external file as a document in the message.
    ///
    /// You can use this to send videos, stickers, audios, or uncompressed photos.
    ///
    /// The Telegram server will be the one that downloads and includes the document as media.
    ///
    /// The text will be the caption of the document, which may be empty for no caption.
    pub fn document_url(mut self, url: impl Into<String>) -> Self {
        self.media = Some(
            (tl::types::InputMediaDocumentExternal {
                spoiler: false,
                url: url.into(),
                ttl_seconds: self.media_ttl,
                video_cover: None,
                video_timestamp: None,
            })
            .into(),
        );
        self
    }

    /// Add additional attributes to the message.
    ///
    /// This must be called *after* setting a file.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn f(client: &mut grammers_client::Client) -> Result<(), Box<dyn std::error::Error>> {
    /// # let audio = client.upload_file("audio.flac").await?;
    /// #
    /// use std::time::Duration;
    /// use grammers_client::{types::Attribute, InputMessage};
    ///
    /// let message = InputMessage::text("").document(audio).attribute(
    ///    Attribute::Audio {
    ///        duration: Duration::new(123, 0),
    ///        title: Some("Hello".to_string()),
    ///        performer: Some("World".to_string()),
    ///    }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn attribute(mut self, attr: Attribute) -> Self {
        if let Some(tl::enums::InputMedia::UploadedDocument(document)) = &mut self.media {
            document.attributes.push(attr.into());
        }
        self
    }

    /// Copy media from an existing message.
    ///
    /// You can use this to send media from another message without re-uploading it.
    pub fn copy_media(mut self, media: &Media) -> Self {
        self.media = media.to_raw_input_media();
        self
    }

    /// Include the uploaded file as a document file in the message.
    ///
    /// You can use this to send any type of media as a simple document file.
    ///
    /// The text will be the caption of the file, which may be empty for no caption.
    pub fn file(mut self, file: Uploaded) -> Self {
        let mime_type = self.get_file_mime(&file);
        let file_name = file.name().to_string();
        self.media = Some(
            (tl::types::InputMediaUploadedDocument {
                nosound_video: false,
                force_file: true,
                spoiler: false,
                file: file.raw,
                thumb: None,
                mime_type,
                attributes: vec![(tl::types::DocumentAttributeFilename { file_name }).into()],
                stickers: None,
                ttl_seconds: self.media_ttl,
                video_cover: None,
                video_timestamp: None,
            })
            .into(),
        );
        self
    }

    /// Change the media's Time To Live (TTL).
    ///
    /// For example, this enables you to send a `photo` that can only be viewed for a certain
    /// amount of seconds before it expires.
    ///
    /// Not all media supports this feature.
    ///
    /// This method should be called before setting any media, else it won't have any effect.
    pub fn media_ttl(mut self, seconds: i32) -> Self {
        self.media_ttl = if seconds < 0 { None } else { Some(seconds) };
        self
    }

    /// Change the media's mime type.
    ///
    /// This method will override the mime type that would otherwise be automatically inferred
    /// from the extension of the used file
    ///
    /// If no mime type is set and it cannot be inferred, the mime type will be
    /// "application/octet-stream".
    ///
    /// This method should be called before setting any media, else it won't have any effect.
    pub fn mime_type(mut self, mime_type: &str) -> Self {
        self.mime_type = Some(mime_type.to_string());
        self
    }

    /// Return the mime type string for the given file.
    fn get_file_mime(&self, file: &Uploaded) -> String {
        if let Some(mime) = self.mime_type.as_ref() {
            mime.clone()
        } else if let Some(mime) = mime_guess::from_path(file.name()).first() {
            mime.essence_str().to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Builds a new message using the given plaintext as the message contents.
    pub fn text<T: AsRef<str>>(s: T) -> Self {
        Self {
            text: s.as_ref().to_string(),
            ..Self::default()
        }
    }

    /// Builds a new message from the given markdown-formatted string as the
    /// message contents and entities.
    ///
    /// Note that Telegram only supports a very limited subset of entities:
    /// bold, italic, underline, strikethrough, code blocks, pre blocks and inline links (inline
    /// links with this format `tg://user?id=12345678` will be replaced with inline mentions when
    /// possible).
    #[cfg(feature = "markdown")]
    pub fn markdown<T: AsRef<str>>(s: T) -> Self {
        let (text, entities) = crate::parsers::parse_markdown_message(s.as_ref());
        Self {
            text,
            entities,
            ..Self::default()
        }
    }

    /// Builds a new message from the given HTML-formatted string as the
    /// message contents and entities.
    ///
    /// Note that Telegram only supports a very limited subset of entities:
    /// bold, italic, underline, strikethrough, code blocks, pre blocks and inline links (inline
    /// links with this format `tg://user?id=12345678` will be replaced with inline mentions when
    /// possible).
    #[cfg(feature = "html")]
    pub fn html<T: AsRef<str>>(s: T) -> Self {
        let (text, entities) = crate::parsers::parse_html_message(s.as_ref());
        Self {
            text,
            entities,
            ..Self::default()
        }
    }
}

impl From<&str> for InputMessage {
    fn from(text: &str) -> Self {
        Self::text(text)
    }
}

impl From<String> for InputMessage {
    fn from(text: String) -> Self {
        Self {
            text,
            ..Self::default()
        }
    }
}

impl From<&super::Message> for InputMessage {
    fn from(message: &super::Message) -> Self {
        Self {
            text: message.text().to_owned(),
            entities: message.fmt_entities().cloned().unwrap_or(Vec::new()),
            media: message.media().and_then(|m| m.to_raw_input_media()),
            ..Default::default()
        }
    }
}

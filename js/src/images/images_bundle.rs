use std::collections::HashMap;

pub fn get_images_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();

    // The arguments for include_bytes! have been fixed to match the HashMap keys.
    assets.insert("add-relay.svg".to_string(), include_bytes!("add-relay.svg") as &'static [u8]);
    assets.insert("event-reply-all.svg".to_string(), include_bytes!("event-reply-all.svg") as &'static [u8]);
    assets.insert("gnostr_notif.svg".to_string(), include_bytes!("gnostr_notif.svg") as &'static [u8]);
    assets.insert("logo.svg".to_string(), include_bytes!("logo.svg") as &'static [u8]);
    assets.insert("open-thread.svg".to_string(), include_bytes!("open-thread.svg") as &'static [u8]);
    assets.insert("close-modal.svg".to_string(), include_bytes!("close-modal.svg") as &'static [u8]);
    assets.insert("event-reply.svg".to_string(), include_bytes!("event-reply.svg") as &'static [u8]);
    assets.insert("home-active.svg".to_string(), include_bytes!("home-active.svg") as &'static [u8]);
    assets.insert("message-user.svg".to_string(), include_bytes!("message-user.svg") as &'static [u8]);
    assets.insert("profile-website.svg".to_string(), include_bytes!("profile-website.svg") as &'static [u8]);
    assets.insert("content-warning.svg".to_string(), include_bytes!("content-warning.svg") as &'static [u8]);
    assets.insert("event-share.svg".to_string(), include_bytes!("event-share.svg") as &'static [u8]);
    assets.insert("home.svg".to_string(), include_bytes!("home.svg") as &'static [u8]);
    assets.insert("messages-active.svg".to_string(), include_bytes!("messages-active.svg") as &'static [u8]);
    assets.insert("profile-zap.svg".to_string(), include_bytes!("profile-zap.svg") as &'static [u8]);
    assets.insert("edit-profile.svg".to_string(), include_bytes!("edit-profile.svg") as &'static [u8]);
    assets.insert("explore-active.svg".to_string(), include_bytes!("explore-active.svg") as &'static [u8]);
    assets.insert("icon-maskable.svg".to_string(), include_bytes!("icon-maskable.svg") as &'static [u8]);
    assets.insert("messages.svg".to_string(), include_bytes!("messages.svg") as &'static [u8]);
    assets.insert("pubkey.svg".to_string(), include_bytes!("pubkey.svg") as &'static [u8]);
    assets.insert("event-delete.svg".to_string(), include_bytes!("event-delete.svg") as &'static [u8]);
    assets.insert("explore.svg".to_string(), include_bytes!("explore.svg") as &'static [u8]);
    assets.insert("icon.icns".to_string(), include_bytes!("icon.icns") as &'static [u8]);
    assets.insert("new-note.svg".to_string(), include_bytes!("new-note.svg") as &'static [u8]);
    assets.insert("read-more.svg".to_string(), include_bytes!("read-more.svg") as &'static [u8]);
    assets.insert("event-details.svg".to_string(), include_bytes!("event-details.svg") as &'static [u8]);
    assets.insert("favicon-notif.ico".to_string(), include_bytes!("favicon-notif.ico") as &'static [u8]);
    assets.insert("icon.svg".to_string(), include_bytes!("icon.svg") as &'static [u8]);
    assets.insert("no-user.svg".to_string(), include_bytes!("no-user.svg") as &'static [u8]);
    assets.insert("settings-active.svg".to_string(), include_bytes!("settings-active.svg") as &'static [u8]);
    assets.insert("event-like.svg".to_string(), include_bytes!("event-like.svg") as &'static [u8]);
    assets.insert("favicon.ico".to_string(), include_bytes!("favicon.ico") as &'static [u8]);
    assets.insert("key.svg".to_string(), include_bytes!("key.svg") as &'static [u8]);
    assets.insert("notifications-active.svg".to_string(), include_bytes!("notifications-active.svg") as &'static [u8]);
    assets.insert("settings.svg".to_string(), include_bytes!("settings.svg") as &'static [u8]);
    assets.insert("event-liked.svg".to_string(), include_bytes!("event-liked.svg") as &'static [u8]);
    assets.insert("gnostr-nobg.svg".to_string(), include_bytes!("gnostr-nobg.svg") as &'static [u8]);
    assets.insert("loader-fragment.svg".to_string(), include_bytes!("loader-fragment.svg") as &'static [u8]);
    assets.insert("notifications.svg".to_string(), include_bytes!("notifications.svg") as &'static [u8]);
    assets.insert("sign-out.svg".to_string(), include_bytes!("sign-out.svg") as &'static [u8]);
    assets.insert("event-options.svg".to_string(), include_bytes!("event-options.svg") as &'static [u8]);
    assets.insert("gnostr.svg".to_string(), include_bytes!("gnostr.svg") as &'static [u8]);
    assets.insert("logo-inverted.svg".to_string(), include_bytes!("logo-inverted.svg") as &'static [u8]);
    assets.insert("open-thread-here.svg".to_string(), include_bytes!("open-thread-here.svg") as &'static [u8]);
    assets.insert("open-thread-here.svg".to_string(), include_bytes!("open-thread-here.svg") as &'static [u8]);
    assets.insert("icon-256.png".to_string(), include_bytes!("../pwa/icon-256.png") as &'static [u8]);
    assets}

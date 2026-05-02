use std::collections::HashMap;

#[cfg(gnostr_workspace_assets)]
pub fn get_images_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();

    assets.insert(
        "add-relay.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/add-relay.svg"))
            as &'static [u8],
    );
    assets.insert(
        "event-reply-all.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/event-reply-all.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "gnostr_notif.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/gnostr_notif.svg"))
            as &'static [u8],
    );
    assets.insert(
        "logo.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/logo.svg"))
            as &'static [u8],
    );
    assets.insert(
        "open-thread.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/open-thread.svg"))
            as &'static [u8],
    );
    assets.insert(
        "close-modal.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/close-modal.svg"))
            as &'static [u8],
    );
    assets.insert(
        "event-reply.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/event-reply.svg"))
            as &'static [u8],
    );
    assets.insert(
        "home-active.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/home-active.svg"))
            as &'static [u8],
    );
    assets.insert(
        "message-user.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/message-user.svg"))
            as &'static [u8],
    );
    assets.insert(
        "profile-website.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/profile-website.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "content-warning.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/content-warning.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "event-share.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/event-share.svg"))
            as &'static [u8],
    );
    assets.insert(
        "home.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/home.svg"))
            as &'static [u8],
    );
    assets.insert(
        "messages-active.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/messages-active.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "profile-zap.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/profile-zap.svg"))
            as &'static [u8],
    );
    assets.insert(
        "edit-profile.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/edit-profile.svg"))
            as &'static [u8],
    );
    assets.insert(
        "explore-active.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/explore-active.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "icon-maskable.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/icon-maskable.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "messages.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/messages.svg"))
            as &'static [u8],
    );
    assets.insert(
        "pubkey.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/pubkey.svg"))
            as &'static [u8],
    );
    assets.insert(
        "event-delete.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/event-delete.svg"))
            as &'static [u8],
    );
    assets.insert(
        "explore.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/explore.svg"))
            as &'static [u8],
    );
    assets.insert(
        "icon.icns".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/icon.icns"))
            as &'static [u8],
    );
    assets.insert(
        "new-note.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/new-note.svg"))
            as &'static [u8],
    );
    assets.insert(
        "read-more.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/read-more.svg"))
            as &'static [u8],
    );
    assets.insert(
        "event-details.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/event-details.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "favicon-notif.ico".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/favicon-notif.ico"
        )) as &'static [u8],
    );
    assets.insert(
        "icon.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/icon.svg"))
            as &'static [u8],
    );
    assets.insert(
        "no-user.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/no-user.svg"))
            as &'static [u8],
    );
    assets.insert(
        "settings-active.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/settings-active.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "event-like.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/event-like.svg"))
            as &'static [u8],
    );
    assets.insert(
        "favicon.ico".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/favicon.ico"))
            as &'static [u8],
    );
    assets.insert(
        "key.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/key.svg"))
            as &'static [u8],
    );
    assets.insert(
        "notifications-active.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/notifications-active.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "settings.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/settings.svg"))
            as &'static [u8],
    );
    assets.insert(
        "event-liked.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/event-liked.svg"))
            as &'static [u8],
    );
    assets.insert(
        "gnostr-nobg.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/gnostr-nobg.svg"))
            as &'static [u8],
    );
    assets.insert(
        "loader-fragment.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/loader-fragment.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "notifications.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/notifications.svg"))
            as &'static [u8],
    );
    assets.insert(
        "sign-out.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/sign-out.svg"))
            as &'static [u8],
    );
    assets.insert(
        "event-options.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/event-options.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "gnostr.svg".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/images/gnostr.svg"))
            as &'static [u8],
    );
    assets.insert(
        "logo-inverted.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/logo-inverted.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "open-thread-here.svg".to_string(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../web/src/images/open-thread-here.svg"
        )) as &'static [u8],
    );
    assets.insert(
        "icon-256.png".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/pwa/icon-256.png"))
            as &'static [u8],
    );
    assets
}

#[cfg(not(gnostr_workspace_assets))]
pub fn get_images_assets() -> HashMap<String, &'static [u8]> {
    HashMap::new()
}

struct JSContacts {
    contacts_js: &'static [u8],
}

impl JSContacts {
    fn new() -> Self {
        let contacts_js_bytes: &'static [u8] = include_bytes!("contacts.js");
        JSContacts {
            contacts_js: contacts_js_bytes,
        }
    }

    fn contacts_js(&self) -> &'static [u8] {
        self.contacts_js
    }

    fn to_string(&self) -> String {
        if let Ok(contacts_js_string) = String::from_utf8(self.contacts_js.to_vec()) {
            contacts_js_string
        } else {
            String::from("js/contacts.js is not valid UTF-8.")
        }
    }
}

//! Construct and manipulate an HTML element represented using Rust types.
//!
//! Conceptually, an element has a tag, optional attributes, and
//! optional children. A child can consist of non-HTML text, or be an
//! element of its own.
//!
//! This crate aims to follow the WhatWG specification at
//! <https://html.spec.whatwg.org/>.
//!
//! # Example
//!
//! ~~~
//! use html_page::{Element, Tag};
//!
//! let e = Element::new(Tag::P)
//!     .with_text("hello ")
//!     .with_child(Element::new(Tag::Strong).with_text("world"));
//! assert_eq!(e.serialize(), "<P>hello <STRONG>world</STRONG></P>");
//! assert_eq!(e.plain_text(), "hello world");
//! ~~~

#![deny(missing_docs)]

use html_escape::{encode_double_quoted_attribute, encode_safe};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// An HTML document ("page'),consisting of a head and a body element.
///
/// ~~~
/// # use html_page::{HtmlPage, Element, Tag};
/// let title = Element::new(Tag::Title).with_text("my page");
/// let doc = HtmlPage::default().with_head_element(title);
/// assert_eq!(format!("{}", doc), "<!DOCTYPE html>\n<HTML>\n\
/// <HEAD><TITLE>my page</TITLE></HEAD>\n<BODY></BODY>\n</HTML>\n");
/// ~~~
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlPage {
    head: Element,
    body: Element,
}

impl Default for HtmlPage {
    fn default() -> Self {
        Self {
            head: Element::new(Tag::Head),
            body: Element::new(Tag::Body),
        }
    }
}

impl HtmlPage {
    /// Append an element to the head.
    pub fn push_to_head(&mut self, e: Element) {
        self.head.push_child(e);
    }

    /// Append an element to the body.
    pub fn push_to_body(&mut self, e: Element) {
        self.body.push_child(e);
    }

    /// Append an element to the head, when constructing.
    pub fn with_head_element(mut self, e: Element) -> Self {
        self.head.push_child(e);
        self
    }

    /// Append an element to the body, when constructing.
    pub fn with_body_element(mut self, e: Element) -> Self {
        self.body.push_child(e);
        self
    }

    /// Append text to the body, when constructing.
    pub fn with_body_text(mut self, text: &str) -> Self {
        self.body.push_text(text);
        self
    }

    /// Append all children of `e` as to body of page.
    pub fn push_children(&mut self, e: &Element) {
        for child in &e.children {
            self.body.children.push(child.clone());
        }
    }
}

impl Display for HtmlPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "<!DOCTYPE html>")?;
        writeln!(f, "<{}>", Tag::Html)?;
        writeln!(f, "{}", &self.head)?;
        writeln!(f, "{}", &self.body)?;
        writeln!(f, "</{}>", Tag::Html)?;
        Ok(())
    }
}

/// The tag of an HTML5 element.
///
/// Note that we only support HTML5 elements, as listed on
/// <https://html.spec.whatwg.org//>.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(missing_docs)] // the variants are just element names, no need
                       // to document each separately
pub enum Tag {
    A,
    Abbr,
    Address,
    Area,
    Article,
    Aside,
    Audio,
    B,
    Base,
    Bdi,
    Bdo,
    Blockquote,
    Body,
    Br,
    Button,
    Canvas,
    Caption,
    Cite,
    Code,
    Col,
    ColGroup,
    Data,
    DataList,
    Dd,
    Del,
    Details,
    Dfn,
    Dialog,
    Div,
    Dl,
    Dt,
    Em,
    Embed,
    FieldSet,
    FigCaption,
    Figure,
    Footer,
    Form,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Head,
    Header,
    Hr,
    Html,
    I,
    Iframe,
    Img,
    Input,
    Ins,
    Kbd,
    Label,
    Legend,
    Li,
    Link,
    Main,
    Map,
    Mark,
    Meta,
    Meter,
    Nav,
    NoScript,
    Object,
    Ol,
    OptGroup,
    Option,
    Output,
    P,
    Param,
    Picture,
    Pre,
    Progress,
    Q,
    Rp,
    Rt,
    Ruby,
    S,
    Samp,
    Script,
    Section,
    Select,
    Small,
    Source,
    Span,
    Strong,
    Style,
    Sub,
    Summary,
    Sup,
    Svg,
    Table,
    Tbody,
    Td,
    Template,
    TextArea,
    Tfoot,
    Th,
    Time,
    Title,
    Tr,
    Track,
    U,
    Ul,
    Var,
    Video,
    Wbr,
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

impl Tag {
    fn as_str(&self) -> &str {
        match self {
            Self::A => "A",
            Self::Abbr => "ABBR",
            Self::Address => "ADDRESS",
            Self::Area => "AREA",
            Self::Article => "ARTICLE",
            Self::Aside => "ASIDE",
            Self::Audio => "AUDIO",
            Self::B => "B",
            Self::Base => "BASE",
            Self::Bdi => "BDI",
            Self::Bdo => "BDO",
            Self::Blockquote => "BLOCKQUOTE",
            Self::Body => "BODY",
            Self::Br => "BR",
            Self::Button => "BUTTON",
            Self::Canvas => "CANVAS",
            Self::Caption => "CAPTION",
            Self::Cite => "CITE",
            Self::Code => "CODE",
            Self::Col => "COL",
            Self::ColGroup => "COLGROUP",
            Self::Data => "DATA",
            Self::DataList => "DATALIST",
            Self::Dd => "DD",
            Self::Del => "DEL",
            Self::Details => "DETAILS",
            Self::Dfn => "DFN",
            Self::Dialog => "DIALOG",
            Self::Div => "DIV",
            Self::Dl => "DL",
            Self::Dt => "DT",
            Self::Em => "EM",
            Self::Embed => "EMBED",
            Self::FieldSet => "FIELDSET",
            Self::FigCaption => "FIGCAPTIO",
            Self::Figure => "FIGURE",
            Self::Footer => "FOOTER",
            Self::Form => "FORM",
            Self::H1 => "H1",
            Self::H2 => "H2",
            Self::H3 => "H3",
            Self::H4 => "H4",
            Self::H5 => "H5",
            Self::H6 => "H6",
            Self::Head => "HEAD",
            Self::Header => "HEADER",
            Self::Hr => "HR",
            Self::Html => "HTML",
            Self::I => "I",
            Self::Iframe => "IFRAME",
            Self::Img => "IMG",
            Self::Input => "INPUT",
            Self::Ins => "INS",
            Self::Kbd => "KBD",
            Self::Label => "LABEL",
            Self::Legend => "LEGEND",
            Self::Li => "LI",
            Self::Link => "LINK",
            Self::Main => "MAIN",
            Self::Map => "MAP",
            Self::Mark => "MARK",
            Self::Meta => "META",
            Self::Meter => "METER",
            Self::Nav => "NAV",
            Self::NoScript => "NOSCRIPT",
            Self::Object => "OBJECT",
            Self::Ol => "OL",
            Self::OptGroup => "OPTGROUP",
            Self::Option => "OPTION",
            Self::Output => "OUTPUT",
            Self::P => "P",
            Self::Param => "PARAM",
            Self::Picture => "PICTURE",
            Self::Pre => "PRE",
            Self::Progress => "PROGRESS",
            Self::Q => "Q",
            Self::Rp => "RP",
            Self::Rt => "RT",
            Self::Ruby => "RUBY",
            Self::S => "S",
            Self::Samp => "SAMP",
            Self::Script => "SCRIPT",
            Self::Section => "SECTION",
            Self::Select => "SELECT",
            Self::Small => "SMALL",
            Self::Source => "SOURCE",
            Self::Span => "SPAN",
            Self::Strong => "STRONG",
            Self::Style => "STYLE",
            Self::Sub => "SUB",
            Self::Summary => "SUMMARY",
            Self::Sup => "SUP",
            Self::Svg => "SVG",
            Self::Table => "TABLE",
            Self::Tbody => "TBODY",
            Self::Td => "TD",
            Self::Template => "TEMPLATE",
            Self::TextArea => "TEXTAREA",
            Self::Tfoot => "TFOOT",
            Self::Th => "TH",
            Self::Time => "TIME",
            Self::Title => "TITLE",
            Self::Tr => "TR",
            Self::Track => "TRACK",
            Self::U => "U",
            Self::Ul => "UL",
            Self::Var => "VAR",
            Self::Video => "VIDEO",
            Self::Wbr => "WBR",
        }
    }

    fn can_self_close(&self) -> bool {
        match self {
            Self::Area
            | Self::Base
            | Self::Br
            | Self::Col
            | Self::Embed
            | Self::Hr
            | Self::Img
            | Self::Input
            | Self::Link
            | Self::Meta
            | Self::Param
            | Self::Source
            | Self::Track
            | Self::Wbr => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test_tag {
    use super::Tag;

    #[test]
    fn can_self_close() {
        assert!(Tag::Area.can_self_close());
        assert!(Tag::Base.can_self_close());
        assert!(Tag::Br.can_self_close());
        assert!(Tag::Col.can_self_close());
        assert!(Tag::Embed.can_self_close());
        assert!(Tag::Hr.can_self_close());
        assert!(Tag::Img.can_self_close());
        assert!(Tag::Input.can_self_close());
        assert!(Tag::Link.can_self_close());
        assert!(Tag::Meta.can_self_close());
        assert!(Tag::Param.can_self_close());
        assert!(Tag::Source.can_self_close());
        assert!(Tag::Track.can_self_close());
        assert!(Tag::Wbr.can_self_close());
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Attributes {
    attrs: HashMap<String, AttributeValue>,
}

impl Attributes {
    fn set(&mut self, name: &str, value: &str) {
        self.attrs
            .insert(name.into(), AttributeValue::String(value.into()));
    }

    fn set_boolean(&mut self, name: &str) {
        self.attrs.insert(name.into(), AttributeValue::Boolean);
    }

    fn unset(&mut self, name: &str) {
        self.attrs.remove(name);
    }

    fn get(&self, name: &str) -> Option<&AttributeValue> {
        self.attrs.get(name)
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        self.attrs.keys().map(|s| s.as_ref())
    }
}

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (name, value) in self.attrs.iter() {
            match value {
                AttributeValue::Boolean => write!(f, " {}", name)?,
                AttributeValue::String(s) => {
                    write!(f, " {}=\"{}\"", name, encode_double_quoted_attribute(s))?
                }
            }
        }
        Ok(())
    }
}

/// The value of an element attribute.
///
/// Attributes may be "boolean" (just the name of an attribute), or a
/// key/value pair, where the value is a string. Technically, in HTML,
/// a boolean attribute with a true value can be expressed as a
/// key/value pair with a value that is an empty string or the name of
/// the attribute, but in this representation we make it more
/// explicit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeValue {
    /// The value of a key/value attribute.
    String(String),
    /// A boolean attribute. It the attribute is present, the value is
    /// true.
    Boolean,
}

impl AttributeValue {
    /// Return value of an attribute as a string. For a boolean
    /// attribute, this is the empty string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s,
            Self::Boolean => "",
        }
    }
}

/// An HTML element.
///
/// The element has a [`Tag`], possibly some attributes, and possibly
/// some children. It may also have a location: this is used when the
/// element is constructed by parsing some input value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Element {
    loc: Option<(usize, usize)>,
    tag: Tag,
    attrs: Attributes,
    children: Vec<Content>,
}

impl Element {
    /// Create a new element, with a given tag.
    pub fn new(tag: Tag) -> Self {
        Self {
            tag,
            attrs: Attributes::default(),
            children: vec![],
            loc: None,
        }
    }

    /// Set the location of an element in a source file.
    pub fn with_location(mut self, line: usize, col: usize) -> Self {
        self.loc = Some((line, col));
        self
    }

    /// Append a child element, when constructing.
    pub fn with_child(mut self, child: Element) -> Self {
        self.children.push(Content::Element(child));
        self
    }

    /// Append a text child, when constructing.
    pub fn with_text(mut self, child: &str) -> Self {
        self.children.push(Content::Text(child.into()));
        self
    }

    /// Set an attribute when creating an element.
    pub fn with_attribute(mut self, name: &str, value: &str) -> Self {
        self.attrs.set(name, value);
        self
    }

    /// Set a boolean attribute when creating an element.
    pub fn with_boolean_attribute(mut self, name: &str) -> Self {
        self.attrs.set_boolean(name);
        self
    }

    /// Add a class when creating an element.
    pub fn with_class(mut self, class: &str) -> Self {
        self.add_class(class);
        self
    }

    /// Return the [`Tag`] of the element.
    pub fn tag(&self) -> Tag {
        self.tag
    }

    /// Return the location of the element.
    pub fn location(&self) -> Option<(usize, usize)> {
        self.loc
    }

    /// Return an iterator over the names of the attributes of an element.
    pub fn attributes(&self) -> impl Iterator<Item = &str> {
        self.attrs.names()
    }

    /// Return the value of an attribute, if the attribute is set.
    /// Otherwise, return `None`.
    pub fn attribute(&self, name: &str) -> Option<&AttributeValue> {
        self.attrs.get(name)
    }

    /// Return the value of an attribute as text, if the attribute is
    /// set. Otherwise, return `None`.
    pub fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(|v| v.as_str())
    }

    /// Set a key/value attribute. If the attribute was already set,
    /// change the value it has.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attrs.set(name, value);
    }

    /// Set a boolean attribute.
    pub fn set_boolean_attribute(&mut self, name: &str) {
        self.attrs.set_boolean(name);
    }

    /// Remove an attribute, which can be key/value or boolean.
    pub fn unset_attribute(&mut self, name: &str) {
        self.attrs.unset(name);
    }

    /// Return current classes set directly for this element.
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        let v = if let Some(v) = self.attribute_value("class") {
            v
        } else {
            ""
        };
        v.split_ascii_whitespace()
    }

    /// Does the element have a class set directly?
    pub fn has_class(&self, wanted: &str) -> bool {
        self.classes().any(|v| v == wanted)
    }

    /// Add a class to the element. This does not replace existing
    /// classes.
    pub fn add_class(&mut self, class: &str) {
        if let Some(old) = self.attribute_value("class") {
            if !old.split_ascii_whitespace().any(|s| s == class) {
                self.set_attribute("class", &format!("{old} {class}"));
            }
        } else {
            self.set_attribute("class", class);
        }
    }

    /// Append text to element. It will be escaped, if needed, when
    /// the element is serialized.
    pub fn push_text(&mut self, text: &str) {
        self.children.push(Content::text(text));
    }

    /// Append a child element to this element.
    pub fn push_child(&mut self, child: Element) {
        self.children.push(Content::element(&child));
    }

    /// Remove all children.
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Append HTML to element. It will NOT be escaped, when the
    /// element is serialized. This is an easy to inject arbitrary
    /// junk into the HTML. No validation is done. You should avoid
    /// this if you can.
    pub fn push_html(&mut self, html: &str) {
        self.children.push(Content::html(html));
    }

    /// Serialize an element into HTML.
    pub fn serialize(&self) -> String {
        format!("{}", self)
    }

    /// Return all the textual content in an element and its children.
    /// This does not include attributes.
    pub fn plain_text(&self) -> String {
        let mut text = TextVisitor::default();
        text.visit(self);
        text.text
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        if self.tag().can_self_close() && self.children.is_empty() {
            write!(f, "<{}{}/>", self.tag, self.attrs)?;
        } else {
            write!(f, "<{}{}>", self.tag, self.attrs)?;
            for child in &self.children {
                write!(f, "{}", child)?;
            }
            write!(f, "</{}>", self.tag)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_element {
    use super::{Element, Tag};

    #[test]
    fn empty_p() {
        let e = Element::new(Tag::P);
        assert_eq!(e.to_string(), "<P></P>");
    }

    #[test]
    fn empty_br() {
        let e = Element::new(Tag::Br);
        assert_eq!(e.to_string(), "<BR/>");
    }
}

/// Represent content in HTML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Content {
    /// Non-HTML text.
    Text(String),
    /// An HTML element.
    Element(Element),
    /// HTML text.
    Html(String),
}

impl Content {
    /// Create a new [`Content::Text`].
    pub fn text(s: &str) -> Self {
        Self::Text(s.into())
    }

    /// Create a new [`Content::Element`].
    pub fn element(e: &Element) -> Self {
        Self::Element(e.clone())
    }

    /// Create a new [`Content::Html`].
    pub fn html(s: &str) -> Self {
        Self::Html(s.into())
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Text(s) => write!(f, "{}", encode_safe(s))?,
            Self::Element(e) => write!(f, "{}", e)?,
            Self::Html(s) => write!(f, "{}", s)?,
        }
        Ok(())
    }
}

/// A read-only visitor for an HTML element.
///
/// Implementing this trait allows "visiting" element and all of its
/// children. The provided [`Visitor::visit`] method visits the
/// element first, and then each of its children in order, and
/// recursively visits the children of each child.
///
/// ~~~
/// # use html_page::{Element, Tag, Visitor};
/// #[derive(Default)]
/// struct Collector {
///     tags: Vec<Tag>,
///     text: String,
/// }
///
/// impl Visitor for Collector {
///     fn visit_element(&mut self, e: &Element) {
///         self.tags.push(e.tag());
///     }
///
///     fn visit_text(&mut self, s: &str) {
///         self.text.push_str(s);
///     }
/// }
/// #
/// # let mut e = Element::new(Tag::P);
/// # e.push_text("hello ");
/// # let mut world = Element::new(Tag::B);
/// # world.push_text("world");
/// # e.push_child(world);
/// #
/// # let mut collector = Collector::default();
/// # collector.visit(&e);
/// # assert_eq!(collector.tags, vec![Tag::P, Tag::B]);
/// # assert_eq!(collector.text, "hello world");
/// ~~~
pub trait Visitor {
    /// Visit an element.
    fn visit_element(&mut self, _: &Element) {}
    /// Visit non-HTML text content.
    fn visit_text(&mut self, _: &str) {}
    /// Visit literal HTML content.
    fn visit_html(&mut self, _: &str) {}

    /// Visit recursively an element and each of its children.
    fn visit(&mut self, root: &Element) {
        self.visit_element(root);
        for child in &root.children {
            match child {
                Content::Text(s) => self.visit_text(s),
                Content::Element(e) => self.visit(e),
                Content::Html(s) => self.visit_html(s),
            }
        }
    }
}

/// A visitor to extract the text of an element and its children.
///
/// This does not include attributes or their values.
///
/// Note that you can call [`Element::plain_text`] for simplicity.
///
/// ~~~
/// use html_page::{Element, Tag, TextVisitor, Visitor};
/// let e = Element::new(Tag::P).with_text("hello, there");
/// let mut tv = TextVisitor::default();
/// tv.visit(&e);
/// assert_eq!(tv.text, "hello, there");
/// ~~~
#[derive(Debug, Default)]
pub struct TextVisitor {
    /// The text collected by the visitor.
    pub text: String,
}

impl Visitor for TextVisitor {
    fn visit_text(&mut self, s: &str) {
        self.text.push_str(s);
    }
}

#[cfg(test)]
mod test {
    use super::{AttributeValue, Content, Element, Tag, Visitor};

    #[test]
    fn element_has_correct_tag() {
        let e = Element::new(Tag::P);
        assert_eq!(e.tag(), Tag::P);
    }

    #[test]
    fn element_has_no_attributes_initially() {
        let e = Element::new(Tag::P);
        assert_eq!(e.attributes().count(), 0);
    }

    #[test]
    fn element_returns_no_value_for_missing_attribute() {
        let e = Element::new(Tag::P);
        assert_eq!(e.attribute("foo"), None);
    }

    #[test]
    fn can_add_attribute_to_element() {
        let mut e = Element::new(Tag::P);
        e.set_attribute("foo", "bar");
        assert_eq!(
            e.attribute("foo"),
            Some(&AttributeValue::String("bar".into()))
        );
        assert_eq!(e.attribute("foo").map(|x| x.as_str()), Some("bar"));
        assert_eq!(e.attribute_value("foo"), Some("bar"));
    }

    #[test]
    fn can_create_element_with_attribute() {
        let e = Element::new(Tag::P).with_attribute("foo", "bar");
        assert_eq!(
            e.attribute("foo"),
            Some(&AttributeValue::String("bar".into()))
        );
    }

    #[test]
    fn can_add_class_to_element() {
        let mut e = Element::new(Tag::P);
        e.add_class("foo");
        let classes: Vec<&str> = e.classes().collect();
        assert_eq!(classes, ["foo"]);
        assert_eq!(e.to_string(), r#"<P class="foo"></P>"#);
    }

    #[test]
    fn can_two_classes_to_element() {
        let mut e = Element::new(Tag::P);
        e.add_class("foo");
        e.add_class("bar");
        let classes: Vec<&str> = e.classes().collect();
        assert_eq!(classes, ["foo", "bar"]);
        assert_eq!(e.to_string(), r#"<P class="foo bar"></P>"#);
    }

    #[test]
    fn can_add_same_class_twice_to_element() {
        let mut e = Element::new(Tag::P);
        e.add_class("foo");
        e.add_class("foo");
        let classes: Vec<&str> = e.classes().collect();
        assert_eq!(classes, ["foo"]);
        assert_eq!(e.to_string(), r#"<P class="foo"></P>"#);
    }

    #[test]
    fn can_add_boolean_attribute_to_element() {
        let mut e = Element::new(Tag::P);
        e.set_boolean_attribute("foo");
        assert_eq!(e.attribute("foo"), Some(&AttributeValue::Boolean));
    }

    #[test]
    fn can_create_element_with_boolan_attribute() {
        let e = Element::new(Tag::P).with_boolean_attribute("foo");
        assert_eq!(e.attribute("foo"), Some(&AttributeValue::Boolean));
    }

    #[test]
    fn unset_attribute_is_unset() {
        let e = Element::new(Tag::P);
        assert_eq!(e.attribute("foo"), None);
    }

    #[test]
    fn can_unset_attribute_in_element() {
        let mut e = Element::new(Tag::P);
        e.set_attribute("foo", "bar");
        e.unset_attribute("foo");
        assert_eq!(e.attribute("foo"), None);
    }

    #[test]
    fn element_has_no_children_initially() {
        let e = Element::new(Tag::P);
        assert!(e.children.is_empty());
    }

    #[test]
    fn add_child_to_element() {
        let mut e = Element::new(Tag::P);
        let child = Content::text("foo");
        e.push_text("foo");
        assert_eq!(e.children, &[child]);
    }

    #[test]
    fn element_has_no_location_initially() {
        let e = Element::new(Tag::P);
        assert!(e.location().is_none());
    }

    #[test]
    fn element_with_location() {
        let e = Element::new(Tag::P).with_location(1, 2);
        assert_eq!(e.location(), Some((1, 2)));
    }

    #[test]
    fn attribute_can_be_serialized() {
        let mut e = Element::new(Tag::P);
        e.set_attribute("foo", "bar");
        assert_eq!(e.serialize(), "<P foo=\"bar\"></P>");
    }

    #[test]
    fn dangerous_attribute_value_is_esacped() {
        let mut e = Element::new(Tag::P);
        e.set_attribute("foo", "<");
        assert_eq!(e.serialize(), "<P foo=\"&lt;\"></P>");
    }

    #[test]
    fn boolean_attribute_can_be_serialized() {
        let mut e = Element::new(Tag::P);
        e.set_boolean_attribute("foo");
        assert_eq!(e.serialize(), "<P foo></P>");
    }

    #[test]
    fn element_can_be_serialized() {
        let mut e = Element::new(Tag::P);
        e.push_text("hello ");
        let mut world = Element::new(Tag::B);
        world.push_text("world");
        e.push_child(world);
        assert_eq!(e.serialize(), "<P>hello <B>world</B></P>");
    }

    #[test]
    fn dangerous_text_is_escaped() {
        let mut e = Element::new(Tag::P);
        e.push_text("hello <world>");
        assert_eq!(e.serialize(), "<P>hello &lt;world&gt;</P>");
    }

    #[test]
    fn element_has_no_class_initially() {
        let e = Element::new(Tag::P);
        assert_eq!(e.attribute_value("class"), None);
        assert_eq!(e.classes().next(), None);
        assert!(!e.has_class("foo"));
    }

    #[test]
    fn element_adds_first_class() {
        let mut e = Element::new(Tag::P);
        e.add_class("foo");
        assert_eq!(e.attribute_value("class"), Some("foo"));
        assert!(e.has_class("foo"));
    }

    #[test]
    fn element_adds_second_class() {
        let mut e = Element::new(Tag::P);
        e.add_class("foo");
        e.add_class("bar");
        assert_eq!(e.attribute_value("class"), Some("foo bar"));
        assert!(e.has_class("foo"));
        assert!(e.has_class("bar"));
    }

    #[test]
    fn creates_classy_element() {
        let e = Element::new(Tag::P).with_class("foo").with_class("bar");
        assert_eq!(e.attribute_value("class"), Some("foo bar"));
        assert!(e.has_class("foo"));
        assert!(e.has_class("bar"));
    }

    #[derive(Default)]
    struct Collector {
        tags: Vec<Tag>,
        text: String,
    }

    impl Visitor for Collector {
        fn visit_element(&mut self, e: &Element) {
            self.tags.push(e.tag());
        }

        fn visit_text(&mut self, s: &str) {
            self.text.push_str(s);
        }
    }

    #[test]
    fn visits_all_children() {
        let e = Element::new(Tag::P)
            .with_text("hello ")
            .with_child(Element::new(Tag::B).with_text("world"));

        let mut collector = Collector::default();
        collector.visit(&e);
        assert_eq!(collector.tags, vec![Tag::P, Tag::B]);
        assert_eq!(collector.text, "hello world");
    }
}

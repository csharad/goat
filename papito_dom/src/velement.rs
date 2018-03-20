use CowStr;
use indexmap::IndexMap;
use std::fmt::{self, Formatter};
use std::fmt::Display;
#[cfg(target_arch = "wasm32")]
use stdweb::web::Element;
#[cfg(target_arch = "wasm32")]
use events::DOMEvent;
use vnode::VNode;

#[derive(Debug, Eq, PartialEq)]
pub struct ClassString(CowStr);

impl Display for ClassString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, " class=\"{}\"", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Attributes(IndexMap<CowStr, CowStr>);

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (k, v) in self.0.iter() {
            write!(f, " {}=\"{}\"", k, v)?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Eq, PartialEq)]
pub struct Events(Vec<Box<DOMEvent>>);

#[derive(Debug, Eq, PartialEq)]
pub struct VElement {
    tag: CowStr,
    class: Option<ClassString>,
    attrs: Option<Attributes>,
    child: Option<Box<VNode>>,
    is_self_closing: bool,
    #[cfg(target_arch = "wasm32")]
    events: Events,
    #[cfg(target_arch = "wasm32")]
    dom_ref: Option<Element>,
}

impl VElement {
    pub fn new(tag: CowStr, class: Option<ClassString>, attrs: Option<Attributes>, child: Option<VNode>, is_self_closing: bool) -> VElement {
        VElement {
            // TODO: validate tag string first
            tag,
            class,
            attrs,
            child: child.map(|it| Box::new(it)),
            is_self_closing,
            #[cfg(target_arch = "wasm32")]
            events: Events(vec![]),
            #[cfg(target_arch = "wasm32")]
            dom_ref: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn dom_ref(&self) -> Option<&Element> {
        self.dom_ref.as_ref()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_events(&mut self, events: Vec<Box<DOMEvent>>) {
        self.events.0 = events;
    }
}

impl Display for VElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<{}", self.tag)?;
        if let Some(ref class) = self.class {
            write!(f, "{}", class)?;
        }
        if let Some(ref attrs) = self.attrs {
            write!(f, "{}", attrs)?;
        }
        if self.is_self_closing {
            write!(f, ">")
        } else {
            write!(f, ">")?;
            if let Some(ref child) = self.child {
                write!(f, "{}", child)?;
            }
            write!(f, "</{}>", self.tag)
        }
    }
}

impl<A: Into<CowStr>> From<A> for ClassString {
    fn from(item: A) -> Self {
        ClassString(item.into())
    }
}

impl<A, B> From<Vec<(A, B)>> for Attributes where
    A: Into<CowStr>,
    B: Into<CowStr> {
    fn from(item: Vec<(A, B)>) -> Self {
        Attributes(item.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect())
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, VNode, bool)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, VNode, bool)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, Some(item.2), item.3)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, VNode)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, VNode)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, Some(item.2), false)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, bool)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, bool)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, None, item.2)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, None, false)
    }
}

impl<A> From<(A, bool)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, bool)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, None, item.1)
    }
}

impl<A> From<(A, ())> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, ())) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, None, false)
    }
}

impl<A> From<(A, VNode, bool)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, VNode, bool)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, Some(item.1), item.2)
    }
}

impl<A> From<(A, VNode)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, VNode)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, Some(item.1), false)
    }
}

fn split_into_class_and_attrs(mut attrs: Attributes) -> (Option<ClassString>, Option<Attributes>) {
    let class = attrs.0.swap_remove("class").map(|it| it.into());
    (class, if attrs.0.len() == 0 { None } else { Some(attrs) })
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use indexmap::IndexMap;
    use stdweb::web::{Element, document, INode, IElement};
    use vdiff::{DOMPatch, DOMRemove};
    use super::{VElement, ClassString, Attributes, Events};
    use vdiff::DOMReorder;
    use vdiff::NextDOMNode;
    use stdweb::web::Node;

    impl DOMPatch<VElement> for VElement {
        fn patch(&mut self, parent: &Element, old_vnode: Option<&mut VElement>) {
            if let Some(old_vnode) = old_vnode {
                if old_vnode.tag != self.tag {
                    old_vnode.remove(parent);
                    create_new_dom_node(self, parent);
                } else {
                    let el = old_vnode.dom_ref().expect("Older element must have dom_ref")
                        .clone();
                    self.class.patch(&el, old_vnode.class.as_mut());
                    self.attrs.patch(&el, old_vnode.attrs.as_mut());
                    self.child.patch(&el, old_vnode.child.as_mut().map(|it| &mut **it));
                    self.events.patch(&el, Some(&mut old_vnode.events));
                    self.dom_ref = Some(el);
                }
            } else {
                create_new_dom_node(self, parent);
            }
        }
    }

    impl DOMReorder for VElement {
        fn reorder_append(&self, parent: &Element) {
            let dom_ref = self.dom_ref().expect("Cannot append previously non-existent element.");
            parent.append_child(dom_ref);
        }

        fn reorder_before(&self, parent: &Element, next: &Node) {
            parent.insert_before(self.dom_ref().expect("Cannot insert previously non-existent text node."), next)
                .unwrap();
        }
    }

    impl DOMRemove for VElement {
        fn remove(&mut self, parent: &Element) {
            let dom_ref = self.dom_ref.take()
                .expect("Cannot remove non-existent element.");
            // Dismember the events
            self.events.remove(&dom_ref);
            // Remove child and their events
            self.child.as_mut().map(|it| &mut **it).remove(&dom_ref);
            // Lastly remove self
            parent.remove_child(&dom_ref).unwrap();
        }
    }

    fn create_new_dom_node(vel: &mut VElement, parent: &Element) {
        let el_node = document().create_element(&vel.tag).unwrap();
        vel.class.patch(&el_node, None);
        vel.attrs.patch(&el_node, None);
        vel.child.patch(&el_node, None);
        vel.events.patch(&el_node, None);
        parent.append_child(&el_node);
        vel.dom_ref = Some(el_node);
    }

    impl DOMPatch<ClassString> for ClassString {
        fn patch(&mut self, parent: &Element, old_value: Option<&mut ClassString>) {
            if Some(&mut *self) != old_value {
                parent.set_attribute("class", &self.0)
                    .unwrap();
            }
        }
    }

    impl DOMRemove for ClassString {
        fn remove(&mut self, parent: &Element) {
            parent.remove_attribute("class");
        }
    }

    impl DOMPatch<Attributes> for Attributes {
        fn patch(&mut self, parent: &Element, old_vnode: Option<&mut Attributes>) {
            let mut deleted_attrs = old_vnode.map(|it| it.0
                .iter()
                .collect::<IndexMap<_, _>>())
                .unwrap_or(IndexMap::new());
            for (k, v) in self.0.iter() {
                let old_attr_val = deleted_attrs.swap_remove(&k);
                if Some(v) != old_attr_val {
                    parent.set_attribute(&k, &v).unwrap();
                }
            }
            for (k, _) in deleted_attrs.iter() {
                parent.remove_attribute(&k);
            }
        }
    }

    impl DOMRemove for Attributes {
        fn remove(&mut self, parent: &Element) {
            for (k, _) in self.0.iter() {
                parent.remove_attribute(k);
            }
        }
    }

    impl DOMPatch<Events> for Events {
        fn patch(&mut self, parent: &Element, mut old_vnode: Option<&mut Events>) {
            // Remove older events because their is no way for Eq between two events.
            old_vnode.remove(parent);
            for ev in self.0.iter_mut() {
                ev.attach(parent);
            }
        }
    }

    impl DOMRemove for Events {
        fn remove(&mut self, _: &Element) {
            for ev in self.0.iter_mut() {
                ev.detach();
            }
        }
    }

    impl NextDOMNode for VElement {
        fn next_dom_node(&self) -> Option<Node> {
            self.dom_ref.clone().map(|it| it.into())
        }
    }
}
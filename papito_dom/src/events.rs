use stdweb::web::{Element, EventListenerHandle, IEventTarget};
use stdweb::web::event::*;
use stdweb::unstable::TryInto;
use std::marker::PhantomData;
use std::fmt::Debug;
use std::fmt::{Formatter, self};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use stdweb::Reference;

/// Add or remove events from the DOM
pub trait DOMEvent {
    fn event_type(&self) -> &'static str;

    fn attach(&mut self, parent: &Element);

    fn detach(&mut self);
}

/// A wrapper construct to encapsulate all events
pub struct DOMEventListener<T, F> where
    F: FnMut(T) + 'static,
    T: ConcreteEvent {
    event_type: &'static str,
    listener: Option<F>,
    listener_handle: Option<EventListenerHandle>,
    _phantom: PhantomData<T>,
}

impl<T, F> DOMEventListener<T, F> where
    F: FnMut(T) + 'static,
    T: ConcreteEvent {
    pub fn new(listener: F) -> DOMEventListener<T, F> {
        DOMEventListener {
            event_type: T::EVENT_TYPE,
            listener: Some(listener),
            listener_handle: None,
            _phantom: PhantomData,
        }
    }
}

impl<T, F> DOMEvent for DOMEventListener<T, F> where
    F: FnMut(T) + 'static,
    T: ConcreteEvent {
    fn event_type(&self) -> &'static str {
        self.event_type
    }

    fn attach(&mut self, parent: &Element) {
        let listener = self.listener.take()
            .expect("Event listener is either already attached or detached");
        let listener_handle = parent.add_event_listener(listener);
        self.listener_handle = Some(listener_handle);
    }

    fn detach(&mut self) {
        let listener_handle = self.listener_handle.take()
            .expect("Event must be attached for it to detach");
        listener_handle.remove();
    }
}

macro_rules! convert_to_dom_ev_listener {
    ($( $listener:ty ),*) => {
        $(
            impl<F> From<F> for DOMEventListener<$listener, F> where
                F: FnMut($listener) {
                fn from(item: F) -> Self {
                    DOMEventListener::new(item)
                }
            }
        )*
    };
}

convert_to_dom_ev_listener!(
    ClickEvent,
    DoubleClickEvent,
    MouseDownEvent,
    MouseUpEvent,
    MouseMoveEvent,
    KeyPressEvent,
    KeyDownEvent,
    KeyUpEvent,
    ProgressEvent,
    LoadStartEvent,
    LoadEndEvent,
    ProgressLoadEvent,
    ProgressAbortEvent,
    ProgressErrorEvent,
    SocketCloseEvent,
    SocketErrorEvent,
    SocketOpenEvent,
    SocketMessageEvent,
    HashChangeEvent,
    PopStateEvent,
    ChangeEvent,
    ResourceLoadEvent,
    ResourceAbortEvent,
    ResourceErrorEvent,
    ResizeEvent,
    InputEvent,
    ReadyStateChangeEvent,
    FocusEvent,
    BlurEvent
);

impl Debug for DOMEvent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "EventType = \"{}\"", self.event_type())
    }
}

// Implemented because of the requirements on VElement. Could not compare two closures
// so a simple pass through `true`.
impl PartialEq for DOMEvent {
    fn eq(&self, _: &DOMEvent) -> bool {
        true
    }
}

impl Eq for DOMEvent {}

pub struct RenderRequest {
    tx: Sender<bool>,
    rx: Receiver<bool>,
    callback_ref: Reference
}

impl RenderRequest {
    pub fn new<T: Fn() + 'static>(on_send: T) -> RenderRequest {
        let (tx, rx) = channel();
        let callback_ref = js! {
            var callback = @{on_send};
            return callback;
        }.try_into().unwrap();
        RenderRequest {
            rx,
            tx,
            callback_ref
        }
    }

    pub fn sender(&self) -> RenderRequestSender {
        RenderRequestSender {
            tx: self.tx.clone(),
            callback_ref: self.callback_ref.clone()
        }
    }

    pub fn receive(&self) -> bool {
        let received = self.rx.try_iter().collect::<Vec<_>>();
        !received.is_empty()
    }
}

#[derive(Clone)]
pub struct RenderRequestSender {
    tx: Sender<bool>,
    callback_ref: Reference
}

impl RenderRequestSender {
    pub fn send(&self) {
        self.tx.send(true)
            .expect("The receiver of the app is not present which is impossible.");
        js!{ @(no_return)
            @{&self.callback_ref}();
        }
    }
}
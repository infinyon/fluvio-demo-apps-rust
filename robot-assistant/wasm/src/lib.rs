mod utils;

use robot_assistant_msg::BotMsg;
use seed::{prelude::*, *};
use web_sys::Location;
use web_sys::Window;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn wsurl() -> Result<String, JsValue> {
    let window: Window = web_sys::window().expect("no global `window` exists");
    let location: Location = window.location();
    let protocol: String = location.protocol()?;
    let host: String = location.host()?;
    let ws_protocol = if protocol == "https:" {
        "wss://"
    } else {
        "ws://"
    };
    Ok(format!("{}{}/ws/", ws_protocol, host))
}

struct Model {
    open: bool,
    connected: bool,
    editable: bool,
    placeholder: String,
    messages: Vec<BotMsg>,
    web_socket: WebSocket,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let open = true;
    let connected = false;
    let editable = false;
    let placeholder = "Choose an option".to_string();
    let web_socket = WebSocket::builder(wsurl().expect("url"), orders)
        .on_open(|| Msg::WebSocketOpened)
        .on_message(Msg::WebSocketMessage)
        .on_close(|_| Msg::WebSocketClosed)
        .on_error(|| Msg::WebSocketFailed)
        .build_and_open()
        .unwrap();
    let messages = vec![];
    Model {
        open,
        connected,
        editable,
        placeholder,
        messages,
        web_socket,
    }
}

enum Msg {
    OpenDialog,
    CloseDialog,
    ResetSession,
    ScrollTop,
    ClickChoice(String),
    WebSocketMessage(WebSocketMessage),
    WebSocketOpened,
    WebSocketClosed,
    WebSocketFailed,
    KeyDown(web_sys::KeyboardEvent),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::OpenDialog => model.open = true,
        Msg::CloseDialog => model.open = false,
        Msg::ScrollTop => {
            let ib = "ib".get_element().expect("ib");
            ib.set_scroll_top(ib.scroll_height() + ib.client_height());
        }
        Msg::WebSocketMessage(msg) => {
            let bot_msg = msg.json::<BotMsg>().expect("bot msg");
            model.messages.push(bot_msg);
            orders.force_render_now();
            orders.send_msg(Msg::ScrollTop);
        }
        Msg::WebSocketOpened => {
            model.connected = true;
            orders.send_msg(Msg::ResetSession);
        }
        Msg::WebSocketClosed => {
            model.connected = false;
        }
        Msg::WebSocketFailed => {
            console_log!("WebSocketFailed");
            panic!();
        }
        Msg::ResetSession => {
            model.messages = vec![];
            model.placeholder = "Type question here...".to_string();
            model.editable = true;
        }
        Msg::ClickChoice(next) => {
            model.web_socket.send_text(next).expect("sent");
        }
        Msg::KeyDown(event) => {
            let key_code = event.key_code();
            let target = event.target().unwrap();
            let textarea = seed::to_textarea(&target);
            if key_code == 13 {
                event.prevent_default();
                let text = textarea.value();
                model.web_socket.send_text(text).expect("sent");
                textarea.set_value("");
            }
        }
    }
}

fn view_choices(choices: &[robot_assistant_msg::Item]) -> Node<Msg> {
    div![choices.iter().map(|choice| {
        let next: String = choice.next.to_string();
        div![button![
            C!["button"],
            attrs! {
                At::Value => choice.next,
            },
            choice.answer.clone(),
            ev(Ev::Click, |_| Msg::ClickChoice(next))
        ]]
    })]
}

fn view_messages(messages: &[BotMsg]) -> Node<Msg> {
    use robot_assistant_msg::BotMsg::*;
    div![
        C!["msg-body"],
        messages.iter().map(|msg| match msg {
            BotText(html) => div![
                C!["msg-left"],
                div![C!["msg"], Node::from_html(html)],
                div![
                    C!["avatar"],
                    img![C!["bot"], attrs! {At::Src => "img/assistant/bot.svg"}]
                ]
            ],
            UserText(text) => div![C!["msg-right"], div![C!["msg"], text]],
            ChoiceRequest(choices) => view_choices(&choices),
        })
    ]
}

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["assistant"],
        button![
            style! {St::Display => if model.open {"none"}else{"block"}},
            img![attrs! {At::Src => "img/assistant/note.svg"}],
            ev(Ev::Click, |_| Msg::OpenDialog),
        ],
        div![
            C!["chat"],
            style! {St::Display => if model.open {"block"}else{"none"}},
            div![
                C!["header"],
                img![C!["bot"], attrs! {At::Src => "img/assistant/bot.svg"}],
                div![
                    C!["overlay"],
                    div![C![if model.connected {
                        "status on"
                    } else {
                        "status off"
                    }]]
                ],
                span!["Bot Assistant"],
                img![
                    C!["close"],
                    attrs! {At::Src => "img/assistant/close.svg"},
                    ev(Ev::Click, |_| Msg::CloseDialog)
                ],
                img![
                    attrs! {At::Src => "img/assistant/redo.svg"},
                    ev(Ev::Click, |_| Msg::ResetSession)
                ],
            ],
            div![
                C!["body-wrapper"],
                div![
                    C!["inner-body"],
                    attrs! {At::Id => "ib"},
                    view_messages(&model.messages)
                ]
            ],
            div![
                C!["footer"],
                textarea![
                    C!["textareaElement"],
                    attrs! {At::Placeholder => model.placeholder, At::ContentEditable => model.editable},
                    keyboard_ev("keydown", Msg::KeyDown)
                ]
            ],
        ],
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    utils::set_panic_hook();
    App::start("app", init, update, view);
}

extern crate rosc;

#[macro_use]
extern crate lazy_static;

use rosc::OscPacket;
use regex::Regex;

use std::net::UdpSocket;
use std::thread;

use configparser::ini::Ini;

use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::dimen::Vector2;
use speedy2d::Window;
use speedy2d::window::{
    UserEventSender,
    WindowHandler,
    WindowHelper,
    WindowSize,
    WindowCreationOptions,
};
use speedy2d::font::{
    Font,
    TextLayout,
    TextOptions,
};

struct MyWindowHandler {
    cue: String,

    // Configuration
    font_size: f32,
    font_color: speedy2d::color::Color,
    window_color: speedy2d::color::Color,
}

impl WindowHandler<String> for MyWindowHandler
{
    fn on_user_event(&mut self, helper: &mut WindowHelper<String>, user_event: String)
    {
        println!("RX: INFO: Cue fired: '{}'", user_event);
        self.cue = user_event;
        helper.request_redraw();
    }

    fn on_draw(&mut self, helper: &mut WindowHelper<String>, graphics: &mut Graphics2D)
    {
        let bytes = include_bytes!("/System/Library/Fonts/Helvetica.ttc");
        let font = Font::new(bytes).unwrap();

        let block = font.layout_text(&self.cue, self.font_size, TextOptions::new());

        graphics.clear_screen(self.window_color);
        graphics.draw_text((100.0, 100.0), self.font_color, &block);

        // Request that we draw another frame once this one has finished
        helper.request_redraw();
    }
}

fn osc_handler(sock: UdpSocket, user_event_sender: UserEventSender<String>) {
    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                handle_packet(packet, &user_event_sender);
            }
            Err(e) => {
                println!("RX: ERR: Error receiving from socket: {}", e);
                break;
            }
        }
    }
}

fn main() -> Result<(), std::string::String> {
    //
    // Read config
    //
    let mut config = Ini::new();
    let config_map = config.load("cue_printer.ini")?;

    println!("{:#?}", config_map);

    //
    // OSC initialization
    // 
    let sock = UdpSocket::bind(config.get("DEFAULT", "bind_addr").unwrap()).unwrap();
    println!("Listening to {}", config.get("DEFAULT", "bind_addr").unwrap());


    //
    // Window initialization
    //
    let window: Window<String> = Window::new_with_user_events(
        "OSC Cue Monitor",
        WindowCreationOptions::new_windowed(
            WindowSize::PhysicalPixels(Vector2::new(640, 480)),
            None
        )
    ).unwrap();

    //
    // Spawn OSC receiver thread
    //
    let user_event_sender = window.create_user_event_sender();
    thread::spawn(|| {
        osc_handler(sock, user_event_sender);
    });


    //
    // Run main loop
    //
    window.run_loop(
        MyWindowHandler {
            cue: "-".to_string(),
            font_size: config.get("DEFAULT", "font_size").unwrap().parse::<f32>().unwrap(),
            font_color: Color::from_hex_rgb(u32::from_str_radix(&config.get("DEFAULT", "font_color").unwrap(), 16).unwrap()),
            window_color: Color::from_hex_rgb(u32::from_str_radix(&config.get("DEFAULT", "window_color").unwrap(), 16).unwrap()),
        }
    )
}

//
// Handle OSC packet. Do error handling and then pass to vMix.
//
fn handle_packet(packet: OscPacket, user_event_sender: &UserEventSender<String>) {
    match packet {
        OscPacket::Message(msg) => {
            println!("RX: INFO: Received addr {} args {:?}", msg.addr, msg.args);

            lazy_static! {
                static ref RE: Regex = Regex::new(r"^/cue/(\d)/go$").unwrap();
            }
            if RE.is_match(msg.addr.as_str()) {
                let cap = RE.captures(msg.addr.as_str()).unwrap();

                user_event_sender.send_event(cap[1].to_string()).unwrap();
            } else {
                println!("RX: Received unknown message {:?}, ignoring", msg)
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("RX: ERR: Rexeived OSC bundle. OSC bundles currently not supported.  Bundle: {:?}", bundle);
        }
    }
}

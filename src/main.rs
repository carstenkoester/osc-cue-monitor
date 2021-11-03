extern crate rosc;

#[macro_use]
extern crate lazy_static;

use rosc::OscPacket;
use regex::Regex;

use std::fs::read;
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

    // Window dimensions
    window_size: Vector2<u32>,

    // Configuration read from config file
    font: Font,
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

    fn on_resize(&mut self, _helper: &mut WindowHelper<String>, size_pixels: Vector2<u32>)
    {
        self.window_size = size_pixels;
    }

    fn on_draw(&mut self, _helper: &mut WindowHelper<String>, graphics: &mut Graphics2D)
    {
        let block = self.font.layout_text(&self.cue, self.font_size, TextOptions::new());
        let pos_x = (self.window_size.x as f32/2.0)-(block.width()/2.0);
        let pos_y = (self.window_size.y as f32/2.0)-(block.height()/2.0);

        graphics.clear_screen(self.window_color);
        graphics.draw_text((pos_x, pos_y), self.font_color, &block);
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
    config.load("cue_printer.ini")?;

    //
    // Window initialization
    //
    let initial_size = Vector2::new(640, 480);
    let window: Window<String> = Window::new_with_user_events(
        "OSC Cue Monitor",
        WindowCreationOptions::new_windowed(
            WindowSize::PhysicalPixels(initial_size),
            None
        )
    ).unwrap();
    let user_event_sender = window.create_user_event_sender();

    //
    // OSC initialization
    //
    let sock = UdpSocket::bind(config.get("network", "bind_addr").unwrap()).unwrap();
    println!("Listening to {}", config.get("network", "bind_addr").unwrap());

    thread::spawn(|| {
        osc_handler(sock, user_event_sender);
    });


    //
    // Read font
    //
    let font_bytes = read(&config.get("font", "path").unwrap()).unwrap();
    let font = Font::new(&font_bytes).unwrap();

    //
    // Run main loop
    //
    window.run_loop(
        MyWindowHandler {
            cue: "-".to_string(),
            font_size: config.get("font", "size").unwrap().parse::<f32>().unwrap(),
            font_color: Color::from_hex_rgb(u32::from_str_radix(&config.get("font", "color").unwrap(), 16).unwrap()),
            font: font,
            window_color: Color::from_hex_rgb(u32::from_str_radix(&config.get("window", "color").unwrap(), 16).unwrap()),
            window_size: initial_size,
        }
    )
}


//
// Handle OSC packet with minimal error handling
//
fn handle_packet(packet: OscPacket, user_event_sender: &UserEventSender<String>) {
    match packet {
        OscPacket::Message(msg) => {
            println!("RX: INFO: Received addr {} args {:?}", msg.addr, msg.args);

            lazy_static! {
                static ref RE: Regex = Regex::new(r"^/cue/([\d\.]+)/go$").unwrap();
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

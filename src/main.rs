extern crate chrono;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;
extern crate rodio;

use ::std::env;
use::std::fs::File;
use::std::io::BufReader;
use rodio::source::Source;
use rodio::{Decoder, OutputStream, Sink};
use chrono::prelude::*;
use piston::event_loop::{EventSettings, Events};
use piston_window::types::Color;
use piston_window::*;
use std::fmt::format;
use std::fs;
use std::time::Duration;
// use chrono::{DateTime, Local, TimeZone};
use graphics::Context;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{Button, PressEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston_window::Glyphs;
use piston_window::{G2d, TextureSettings};
use std::os::windows;
use std::slice::Windows;
use std::thread;
use std::time::{self, Instant};

pub fn draw_text(
    ctx: &Context,
    graphics: &mut G2d,
    color: Color,
    pos: [f64; 2],
    text: &str,
    glyph: &mut Glyphs,
    font_size: u32
) {
    text::Text::new_color(color, font_size)
        .draw(
            text,
            glyph,
            &ctx.draw_state,
            ctx.transform.trans(pos[0] as f64, pos[1] as f64),
            graphics,
        )
        .unwrap();
}
enum DateTimeComponent {
    Date { Year: i32, Month: u32, Day: u32 },
    Time { Hour: u32, Minute: u32, Second: u32 },
}
impl DateTimeComponent {
    fn new_Time() -> Self {
        let now = Local::now();
        DateTimeComponent::Time {
            Hour: now.hour(),
            Minute: now.minute(),
            Second: now.second(),
        }
    }
    fn new_Date() -> Self {
        let now = Local::now();
        DateTimeComponent::Date {
            Year: now.year(),
            Month: now.month(),
            Day: now.day(),
        }
    }
    fn get_time_string(&self) -> String {
        if let DateTimeComponent::Time {
            Hour,
            Minute,
            Second,
        } = self
        {
            format!("{:02}:{:02}:{:02}", Hour, Minute, Second)
        } else {
            String::new() // Return empty string if it's not `Time` variant
        }
    }
    fn get_time_string_2(&self) -> String {
        if let DateTimeComponent::Time { Hour, Minute, .. } = self {
            format!("{:02}:{:02}", Hour, Minute)
        } else {
            String::new() // Return empty string if it's not `Time` variant
        }
    }
}

struct Alarm {
    Current_time: DateTimeComponent,
    Alarm_time: String,
}
impl Alarm {
    fn new_alarm(current_time: DateTimeComponent, alarm_time: String) -> Self {
        Alarm {
            Current_time: current_time,
            Alarm_time: alarm_time,
        }
    }
    fn get_alarm_string(x: u32, y: u32) -> String {
        format!("{:02}:{:02}", x, y)
    }
    fn check_if_alarm(&mut self, current_time: String) -> bool {
        if current_time == self.Alarm_time {
            return true;
        }
        false
    }
}
struct Timer{
    Hours: u32, 
    Minutes: u32
}
enum Command {
    Clock,
    Alarm(u32, u32),
    Timer(u32, u32),
}
fn main() {
    let opengl = OpenGL::V4_5;
    let mut windows: PistonWindow = WindowSettings::new("Clock", [1000, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut glyphs = Glyphs::new(
        "Assets/DIGITALDREAM.ttf",
        windows.create_texture_context(),
        TextureSettings::new(),
    )
    .unwrap();

    let mut current_time = DateTimeComponent::new_Time();
    let mut last_update = Instant::now();
    let mut time_string = current_time.get_time_string();

    let args: Vec<String> = env::args().collect();

    let command = match args[1].as_str() {
        "clock" => Command::Clock,
        "alarm" => Command::Alarm(
            args[2].parse().expect("Failed to oconvert to integer"),
            args[3].parse().expect("Failed to convert to an integer"),
        ),
       
        _ => panic!("Provide accurate commmand"),
    };
    match command {
        Command::Clock => {
            while let Some(event) = windows.next() {
                if last_update.elapsed() >= Duration::from_secs(1) {
                    current_time = DateTimeComponent::new_Time();
                }
                if let Some(args) = event.render_args() {
                    windows.draw_2d(&event, |context, graphics, device| {
                        let window_width = args.window_size[0] as f64;
                        let window_height = args.window_size[1] as f64;
                        let font_size = 100;
                        time_string = current_time.get_time_string();
                        let text_width = measure_text_width(&time_string, 100, &mut glyphs);
                        clear([0.0, 0.0, 0.0, 1.0], graphics);
                        draw_text(
                            &context,
                            graphics,
                            [1.0, 1.0, 1.0, 1.0],
                            [(window_width - text_width) / 2.0, window_height / 2.0],
                            &time_string,
                            &mut glyphs,
                            font_size,
                        );
                        
                        glyphs.factory.encoder.flush(device);
                    });
                }
            }
        }
        Command::Alarm(hours, minute) => {
            let(_stream, stream_handle) = OutputStream::try_default().unwrap();
            let file = BufReader::new(File::open("Assets/digital-alarm-clock-151920.mp3").unwrap());
            let source = Decoder::new(file).unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let  looped_source = source.repeat_infinite();
            
            
            let mut alarm_time = Alarm::get_alarm_string(hours, minute);
            let mut time_string = current_time.get_time_string_2();

                while let Some(event) = windows.next() {
                    if last_update.elapsed() >= Duration::from_secs(1) {
                        current_time = DateTimeComponent::new_Time();
                    }
                    if alarm_time == time_string {
                        sink.append(looped_source.clone());
                        sink.play();
                    }
                    if let Some(args) = event.render_args() {
                        
                        windows.draw_2d(&event, |context, graphics, device| {
                            let window_width = args.window_size[0] as f64;
                            let window_height = args.window_size[1] as f64;
                            time_string = current_time.get_time_string_2();
                            let text_width = measure_text_width(&time_string, 100, &mut glyphs);
                            let alarm_width = measure_text_width(&time_string, 10, &mut glyphs);
                            let font_size_clock = 100;
                            let font_size_alarm = 10;
                            clear([0.0, 0.0, 0.0, 1.0], graphics);
                            draw_text(
                                &context,
                                graphics,
                                [1.0, 1.0, 1.0, 1.0],
                                [(window_width - text_width) / 2.0, window_height / 2.0],
                                &time_string,
                                &mut glyphs,
                                font_size_clock,
                            );
                            draw_text(
                                &context,
                                graphics,
                                [1.0, 1.0, 1.0, 1.0],
                                [(window_width - alarm_width) / 2.0, (window_height / 2.0) - 100.0],
                                &alarm_time,
                                &mut glyphs,
                                font_size_alarm,
                            );
                            glyphs.factory.encoder.flush(device);
                        });
                    }
                }
            
        }

        _ => (),
    }

    //
    // Alarm::get_alarm_string(11, 33, 40);
    //
    // let mut Alarm = Alarm::new_alarm(alarm_current_time, alarm_time);
}
fn measure_text_width(text: &str, font_size: u32, glyph: &mut Glyphs) -> f64 {
    text.chars()
        .map(|ch| {
            let character = glyph.character(font_size, ch).unwrap();
            character.advance_width()
        })
        .sum()
}

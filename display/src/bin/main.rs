#![no_std]
#![no_main]
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use embedded_graphics::{pixelcolor::Rgb666, pixelcolor::RgbColor, prelude::*};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{self, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use log::info;
use mipidsi::interface::SpiInterface; // Provides the builder for DisplayInterface
use mipidsi::options::ColorOrder;
use mipidsi::{models::ST7789, Builder};
use u8g2_fonts::{fonts, FontRenderer};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let mut display_power = esp_hal::gpio::Output::new(
        peripherals.GPIO7,
        gpio::Level::High,
        OutputConfig::default(),
    );
    display_power.set_high();

    let mut tft_backlight = esp_hal::gpio::Output::new(
        peripherals.GPIO45,
        gpio::Level::High,
        OutputConfig::default(),
    );
    tft_backlight.set_high(); // Turn on the display backlight

    let spi_config = Config::default()
        .with_frequency(Rate::from_mhz(3))
        .with_mode(Mode::_0);

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_sck(peripherals.GPIO36)
        .with_mosi(peripherals.GPIO35)
        .with_miso(peripherals.GPIO37);

    let dc = Output::new(peripherals.GPIO40, Level::High, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO42, Level::High, OutputConfig::default());
    let mut delay = esp_hal::delay::Delay::new();
    let mut buffer = [0_u8; 4096];
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let si = SpiInterface::new(spi_device, dc, &mut buffer);
    let mut display = Builder::new(ST7789, si)
        .color_order(ColorOrder::Rgb)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(Output::new(
            peripherals.GPIO41,
            Level::High,
            OutputConfig::default(),
        ))
        .orientation(
            mipidsi::options::Orientation::default().rotate(mipidsi::options::Rotation::Deg90),
        )
        .init(&mut delay)
        .unwrap();
    display.clear(Rgb666::BLACK.into()).unwrap();
    info!("I don't have anything to read this");
    let train_name = String::from("Test Train\n");
    let suffix = " minutes";
    let mut seconds_delta = 500;
    let mut last_output = String::new();
    let title_font = FontRenderer::new::<fonts::u8g2_font_helvR18_tr>();
    let time_font = FontRenderer::new::<fonts::u8g2_font_helvB24_tr>();

    loop {
        let mins = seconds_delta / 60;
        let seconds = seconds_delta % 60;
        let textual_seconds = if seconds > 30 { ".5" } else { "" };

        let text = mins.to_string() + textual_seconds;

        let text = text + suffix;
        if text != last_output {
            // TODO -- swap to just resetting bounding box for minutes
            display.clear(Rgb666::BLACK.into()).unwrap();
            title_font
                .render_aligned(
                    train_name.as_ref(),
                    Point::new(
                        display.bounding_box().center().x,
                        display.bounding_box().center().y - 40,
                    ),
                    u8g2_fonts::types::VerticalPosition::Baseline,
                    u8g2_fonts::types::HorizontalAlignment::Center,
                    u8g2_fonts::types::FontColor::Transparent(RgbColor::BLUE),
                    &mut display,
                )
                .unwrap();

            time_font
                .render_aligned(
                    text.as_ref(),
                    Point::new(
                        display.bounding_box().center().x,
                        display.bounding_box().center().y + 20,
                    ),
                    u8g2_fonts::types::VerticalPosition::Baseline,
                    u8g2_fonts::types::HorizontalAlignment::Center,
                    u8g2_fonts::types::FontColor::Transparent(RgbColor::WHITE),
                    &mut display,
                )
                .unwrap();

            last_output = text.clone();
        }

        seconds_delta -= 1;

        if seconds_delta < 0 {
            seconds_delta = 500;
        }
        delay.delay_millis(1000);
    }
}

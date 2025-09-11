#![no_std]
#![no_main]
extern crate alloc;

use embedded_graphics::{pixelcolor::Rgb666, prelude::*};
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

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

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
    let mut buffer = [0_u8; 512];
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let si = SpiInterface::new(spi_device, dc, &mut buffer);
    let mut display = Builder::new(ST7789, si)
        .color_order(ColorOrder::Bgr)
        .reset_pin(Output::new(
            peripherals.GPIO41,
            Level::High,
            OutputConfig::default(),
        ))
        .init(&mut delay)
        .unwrap();
    info!("I don't have anything to read this");
    let text = "Test Output";
    let character_style = embedded_graphics::mono_font::MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_6X10,
        embedded_graphics::pixelcolor::RgbColor::RED,
    );
    embedded_graphics::text::Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        embedded_graphics::text::Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    loop {}
}

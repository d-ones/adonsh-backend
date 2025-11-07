#![no_std]
#![no_main]
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use blocking_network_stack::Stack;
use core::net::Ipv4Addr;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{pixelcolor::Rgb565, pixelcolor::RgbColor, prelude::*};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io::{Read, Write};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{self, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::peripherals::{RNG, TIMG0, WIFI};
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};
use esp_wifi::wifi::{ClientConfiguration, Configuration};
use log::info;
use mipidsi::interface::SpiInterface; // Provides the builder for DisplayInterface
use mipidsi::options::ColorOrder;
use mipidsi::{models::ST7789, Builder};
use smoltcp::{
    iface::{SocketSet, SocketStorage},
    wire::{DhcpOption, IpAddress},
};
use u8g2_fonts::{fonts, FontRenderer};

esp_bootloader_esp_idf::esp_app_desc!();

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASS: &str = env!("WIFI_PASS");

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

    display.clear(Rgb565::BLACK.into()).unwrap();

    // Static route name for display (will be used to query endpoint)
    let route_name = String::from("Northbound Express\n");

    // 1. Already ordered Vec of second values for testing -- will come from endpoint
    let mut seconds_deltas: Vec<i32> = Vec::from([
        15,  // 0 min 15s
        120, // 2 min 0s
        185, // 3 min 5s
    ]);

    let mut last_output = String::new();
    let title_font = FontRenderer::new::<fonts::u8g2_font_helvR18_tr>();
    let time_font = FontRenderer::new::<fonts::u8g2_font_helvB24_tr>();

    //WiFi setup
    write_banner(
        &mut display,
        &title_font,
        String::from("Configuring WiFi..."),
        true,
    )
    .unwrap();

    let r = setup_wifi(peripherals.TIMG0, peripherals.RNG, peripherals.WIFI);

    display.clear(Rgb565::BLACK.into()).unwrap();

    if r.is_err() {
        write_banner(
            &mut display,
            &title_font,
            String::from("Error connecting \n to WiFi"),
            false,
        )
        .unwrap();
        loop {}
    };

    write_banner(
        &mut display,
        &title_font,
        String::from("WiFi connected \n successfully"),
        true,
    )
    .unwrap();
    delay.delay_millis(5000);
    display.clear(Rgb565::BLACK.into()).unwrap();

    title_font
        .render_aligned(
            route_name.as_ref(),
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

    loop {
        // Pop if a value is expired and go to the next one
        if let Some(seconds) = seconds_deltas.first() {
            if *seconds < 0 {
                seconds_deltas.remove(0);
                info!("Train departed, remaining: {} times", seconds_deltas.len());
            }
        }

        let seconds_delta = seconds_deltas.first().copied().unwrap_or(0);

        let (_, final_output) = if seconds_deltas.is_empty() {
            ("--".to_string(), "--".to_string())
        } else {
            let mins = seconds_delta / 60;
            let seconds = seconds_delta % 60;
            let textual_seconds = if seconds >= 30 { ".5" } else { "" };

            let suffix = if mins == 1 && seconds < 30 {
                " minute"
            } else {
                " minutes"
            };

            let text = mins.to_string() + textual_seconds;
            let final_output = text.clone() + suffix;

            (text, final_output)
        };

        if final_output != last_output {
            // TODO only clear the bounding box of the old time text later (WIP).
            display.clear(Rgb565::BLACK.into()).unwrap();

            title_font
                .render_aligned(
                    route_name.as_ref(),
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
                    final_output.as_ref(),
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

            last_output = final_output.clone();
        }

        // Decrement all the array deltas
        for seconds in seconds_deltas.iter_mut() {
            *seconds -= 1;
        }
        delay.delay_millis(1000);
    }
}

fn setup_wifi(
    timg0_periph: TIMG0,
    rng_periph: RNG,
    wifi_periph: WIFI,
) -> Result<(), esp_wifi::wifi::WifiError> {
    // 1. Initialize the internal ESP-HAL drivers from the raw peripherals.
    let timg0 = TimerGroup::new(timg0_periph);
    let rng = Rng::new(rng_periph);

    // 2. Run the core esp-wifi initialization.
    let init = esp_wifi::init(
        timg0.timer0, // Pass the TIMER0 driver (moved out of the TimerGroup)
        rng,          // Pass the RNG driver
    )
    .unwrap();
    let (mut controller, interfaces) = esp_wifi::wifi::new(&init, wifi_periph)?;
    let mut device = interfaces.sta;

    let iface = smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&device.mac_address()),
        )),
        &mut device,
        smoltcp::time::Instant::from_micros(
            esp_hal::time::Instant::now()
                .duration_since_epoch()
                .as_micros() as i64,
        ),
    );

    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    dhcp_socket.set_outgoing_options(&[DhcpOption {
        kind: 12,
        data: b"esp-radio",
    }]);
    socket_set.add(dhcp_socket);

    let now = || {
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_millis()
    };
    let seed: u32 = esp_hal::time::Instant::now()
        .duration_since_epoch()
        .as_micros() as u32;

    let stack = Stack::new(iface, device, socket_set, now, seed);

    controller
        .set_power_saving(esp_wifi::config::PowerSaveMode::None)
        .unwrap();

    let client_config = Configuration::Client({
        let mut config = ClientConfiguration::default();
        config.ssid = WIFI_SSID.into();
        config.password = WIFI_PASS.into();
        config
    });

    controller.set_configuration(&client_config).unwrap();
    controller.start()?;
    controller.connect()?;

    // Wait for WiFi connection
    let mut retries = 0;
    loop {
        match controller.is_connected() {
            Ok(true) => break,
            Ok(false) => {
                esp_hal::delay::Delay::new().delay_millis(100);
                retries = retries + 1;
                // poll for 5 seconds
                if retries == 50 {
                    return Err(esp_wifi::wifi::WifiError::NotInitialized);
                }
            }
            Err(e) => return Err(e),
        }
    }

    // Wait for network stack to be ready (DHCP)
    loop {
        stack.work();
        if stack.is_iface_up() {
            break;
        }
    }

    // Make HTTP request
    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut socket = stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    socket.work();
    socket
        .open(IpAddress::Ipv4(Ipv4Addr::new(142, 250, 185, 115)), 80)
        .unwrap();
    socket
        .write(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
        .unwrap();
    socket.flush().unwrap();

    let deadline = esp_hal::time::Instant::now()
        .duration_since_epoch()
        .as_millis()
        + 20_000; // 20 seconds in milliseconds

    let mut buffer = [0u8; 512];
    loop {
        socket.work();
        if let Ok(len) = socket.read(&mut buffer) {
            if len > 0 {
                // Process response here
                // let response = core::str::from_utf8(&buffer[..len]);
            }
        }
        if now() > deadline {
            break;
        }
    }

    socket.disconnect();
    //End WiFi

    Ok(())
}

fn write_banner<D>(
    display: &mut D,
    fr: &FontRenderer,
    s: String,
    info: bool,
) -> Result<
    Option<Rectangle>,
    u8g2_fonts::Error<<D as embedded_graphics::draw_target::DrawTarget>::Error>,
>
where
    D: DrawTarget<Color = Rgb565>,
{
    fr.render_aligned(
        s.as_ref(),
        Point::new(
            display.bounding_box().center().x,
            display.bounding_box().center().y - 40,
        ),
        u8g2_fonts::types::VerticalPosition::Baseline,
        u8g2_fonts::types::HorizontalAlignment::Center,
        u8g2_fonts::types::FontColor::Transparent(match info {
            true => RgbColor::BLUE,
            false => RgbColor::RED,
        }),
        display,
    )
}

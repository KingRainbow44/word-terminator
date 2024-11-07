use std::fmt::Display;
use std::iter::once;
use std::thread::sleep;
use std::time::Duration;
use image::{DynamicImage, RgbImage, RgbaImage};
use log::{error, info};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::*;
use xcap::Window;
use crate::{board, LETTERS};

/// Takes a screenshot of the window at the coordinates.
/// device_name: The name of the device to take a screenshot of. (window name)
pub fn take_screenshot<S: AsRef<str>>(device_name: S) -> anyhow::Result<RgbaImage> {
    let windows = Window::all()?;

    // Find the window.
    let window = windows
        .iter()
        .find(|w| w.title().contains(device_name.as_ref()))
        .ok_or_else(|| anyhow::anyhow!("Window not found."))?;

    // Take a screenshot.
    Ok(window.capture_image()?)
}

/// Processes a bare-bones game board.
/// image: The image to process.
pub fn process_board(image: &RgbImage) -> String {
    // Lock the letters library.
    let letters = LETTERS.read().unwrap();

    // Split the image into a 4x4 grid.
    let mut board = String::new();

    // Crop the board into 4x4 tiles.
    let image = DynamicImage::ImageRgb8(image.clone());
    for row in 0..4 {
        for column in 0..4 {
            let image = board::crop_image(image.clone(), row, column);

            // Determine which letter matches the image.
            let letter = letters.compare(&image);
            board.push(letter);
        }
        board.push('\n');
    }

    board.trim().to_string()
}

/// Finds the window handle by the name of the device.
/// If no window is found, the program terminates with error code 1.
/// device_name: The name of the device to find.
pub unsafe fn get_window<S: AsRef<str> + Display>(device_name: S) -> HWND {
    let name = String::from(device_name.as_ref());
    let name = name
        .encode_utf16()
        .chain(once(0))
        .collect::<Vec<u16>>();

    let mut retries = 0;
    loop {
        match FindWindowW(None, PCWSTR(name.as_ptr())) {
            Ok(handle) => break handle,
            Err(_) if retries < 10 => {
                retries += 1;
                sleep(Duration::from_millis(100));
            },
            _ => {
                error!("Failed to find window handle for '{}'.", device_name);
                std::process::exit(1);
            }
        }
    }
}

/// Sets the size of the window.
/// handle: The handle of the window to resize.
/// width: The new width of the window.
/// height: The new height of the window.
pub unsafe fn set_size(
    handle: HWND,
    width: i32, height: i32,
    (x, y): (i32, i32)
) {
    // Log the current window position if none was specified.
    if (x, y) == (0, 0) {
        let mut rect = RECT::default();
        if let Ok(_) = GetWindowRect(handle, &mut rect) {
            info!("Window is currently at ({}, {})", rect.left, rect.top);
        }
    }

    // Determine the function flags.
    let mut flags = SWP_NOMOVE | SWP_NOZORDER;
    if (x, y) != (0, 0) {
        flags = SWP_NOZORDER;
    }

    // Move/resize the window.
    if let Err(error) = SetWindowPos(handle, HWND_TOP, x, y, width, height, flags) {
        error!("Failed to set the window size: {:?}", error);
        std::process::exit(1);
    }
}

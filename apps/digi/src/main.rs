use std::{sync::Arc, time::Duration};

use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Timer, TimerMode};

fn main() -> anyhow::Result<()> {
    let main_window = Arc::new(MainWindow::new()?);

    let timer = Timer::default();
    {
        let main_window = Arc::clone(&main_window);

        let mut buffer = SharedPixelBuffer::new(256, 256);
        let mut pixels =
            (0..256).flat_map(|r| (0..256).map(move |g| Rgb8Pixel::new(r as u8, g as u8, 0)));
        buffer.make_mut_slice().fill_with(|| pixels.next().unwrap());

        let mut temp = vec![Rgb8Pixel::new(0, 0, 0); 256];
        timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
            temp.copy_from_slice(&buffer.as_slice()[255 * 256..]);
            buffer.make_mut_slice().copy_within(0..255 * 256, 256);
            buffer.make_mut_slice()[..256].copy_from_slice(&temp);
            let image = Image::from_rgb8(buffer.clone());
            main_window.set_map(image);
        });
    }

    main_window.run()?;

    Ok(())
}

slint::slint! {
    export component MainWindow inherits Window {
        in property <image> map;

        Image {
            width: 100%;
            height: 100%;
            image-fit: ImageFit.fill;
            image-rendering: ImageRendering.smooth;
            source: map;
        }
    }
}

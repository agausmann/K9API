use slint::{Image, Rgb8Pixel, SharedPixelBuffer};

fn main() -> anyhow::Result<()> {
    let main_window = MainWindow::new()?;

    let mut buffer = SharedPixelBuffer::new(256, 256);
    let mut pixels =
        (0..256).flat_map(|r| (0..256).map(move |g| Rgb8Pixel::new(r as u8, g as u8, 0)));
    buffer
        .make_mut_slice()
        .fill_with(move || pixels.next().unwrap());
    let image = Image::from_rgb8(buffer);
    main_window.set_map(image);

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

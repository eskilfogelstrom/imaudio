use image::{self, GenericImageView};
use mp3lame_encoder::{Builder, DualPcm, FlushNoGap, Id3Tag, MonoPcm};
use puremp3;
use std::{fs::File, io::Write};
use wav;

fn to_mp3_buffer(input: &[u16]) -> Vec<u16> {
    let mut mp3_encoder = Builder::new().expect("Create LAME builder");
    mp3_encoder.set_num_channels(2).expect("set channels");
    mp3_encoder
        .set_sample_rate(44_100)
        .expect("set sample rate");
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps320)
        .unwrap();
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .unwrap();

    let mut mp3_encoder = mp3_encoder.build().unwrap();

    // let mut buffer: Vec<u8> = vec![0; wave.len()];
    let input_pcm = DualPcm {
        left: &input,
        right: &input,
    };

    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(
        input_pcm.left.len(),
    ));
    let encoded_size = mp3_encoder
        .encode(input_pcm, mp3_out_buffer.spare_capacity_mut())
        .expect("To encode");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let encoded_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
        .expect("to flush");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let (header, samples) = puremp3::read_mp3(&mp3_out_buffer[..]).unwrap();

    let mut out: Vec<u16> = Vec::new();
    for (left, right) in samples {
        let input_min = -1.0;
        let input_max = 1.0;
        let output_max = std::u16::MAX as f32;

        // Normalize the input value
        let normalized_value = (left - input_min) / (input_max - input_min);

        // Scale the normalized value to the output range
        let scaled_value = normalized_value * (output_max as f32);

        // Convert the scaled value to u16
        let result = scaled_value.round() as u16;
        out.push(result);
    }

    return out;
}

fn main() {
    let img = image::open("in.webp").unwrap();

    let rgb = img.to_rgb16();

    let pixels = rgb.enumerate_pixels();

    let mut wave = [Vec::new(), Vec::new(), Vec::new()];

    for (x, y, rgb) in pixels {
        wave[0].push(rgb.0[0] as u16);
        wave[1].push(rgb.0[1] as u16);
        wave[2].push(rgb.0[2] as u16);
    }

    let buffers: [Vec<u16>; 3] = [
        to_mp3_buffer(&wave[0]),
        to_mp3_buffer(&wave[1]),
        to_mp3_buffer(&wave[2]),
    ];

    let mut imgbuf = image::ImageBuffer::new(img.dimensions().0, img.dimensions().1);

    let mut count = 0;
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        *pixel = image::Rgb([buffers[0][count], buffers[1][count], buffers[2][count]]);
        count += 1;
    }

    imgbuf.save("out.png").unwrap();
}

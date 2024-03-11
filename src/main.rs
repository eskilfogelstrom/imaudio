use image::{self, GenericImageView};
use minimp3::{Decoder, Error, Frame};
use mp3lame_encoder::{Builder, FlushNoGap, MonoPcm};

fn to_mp3_buffer(input: &[u16]) -> Vec<u16> {
    let mut mp3_encoder = Builder::new().unwrap();
    mp3_encoder.set_num_channels(1).unwrap();
    mp3_encoder.set_sample_rate(44_100).unwrap();
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps96)
        .unwrap();
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Worst)
        .unwrap();

    let mut mp3_encoder = mp3_encoder.build().unwrap();

    // let mut buffer: Vec<u8> = vec![0; wave.len()];
    let input_pcm = MonoPcm(&input);

    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(input_pcm.0.len()));
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

    let mut out: Vec<u16> = Vec::new();
    let mut decoder = Decoder::new(&mp3_out_buffer[..]);

    loop {
        match decoder.next_frame() {
            Ok(Frame { data, .. }) => {
                for s in data {
                    out.push(s as u16);
                }
            }
            Err(Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    return out;
}

fn main() {
    let img = image::open("pattern.jpeg").unwrap();

    let rgb = img.to_rgba16();

    let pixels = rgb.pixels();

    let mut wave = [Vec::new(), Vec::new(), Vec::new()];

    for p in pixels {
        wave[0].push(p.0[0]);
        wave[1].push(p.0[1]);
        wave[2].push(p.0[2]);
    }

    let buffers = [
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

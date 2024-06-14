use rodio::{Decoder, OutputStream, source::Source};
use std::{fs::{self, File}, io::BufReader};
use rand::seq::SliceRandom;

fn main() {
    // Créer un flux de sortie audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();


    let sound_dir = "sounds";
    let paths: Vec<_> = fs::read_dir(sound_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();


    if paths.is_empty() {
        eprintln!("No sound files found in directory");
        return;
    }


    let random_path = paths.choose(&mut rand::thread_rng()).unwrap();


    let file = File::open(&random_path).expect("Failed to open sound file");
    let source = Decoder::new(BufReader::new(file)).unwrap();


    stream_handle.play_raw(source.convert_samples()).expect("Failed to play sound");


    println!("Musique entendu : {:?}", random_path);

    // Maintenir le programme en vie pour permettre la lecture audio
    std::thread::sleep(std::time::Duration::from_secs(100));
}
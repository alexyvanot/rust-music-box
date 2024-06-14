#![forbid(unsafe_code)]

use regex::Regex;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::exit;
use tempfile::TempDir;
use ytd_rs::{Arg, YoutubeDL};

fn main() {
    // Flag pour contrôler la sortie du programme
    let mut should_exit = false;
    
    // Création d'un répertoire temporaire
    let tmp_dir = TempDir::new().unwrap_or_else(|err| {
        eprintln!("Error occurred while creating temporary directory: {err}");
        exit(1)
    });

    // Gestion du signal Ctrl-C
    ctrlc::set_handler(move || {}).expect("Error setting Ctrl-C handler");

    // Expression régulière pour détecter les liens YouTube
    let youtube_regex = Regex::new(
        r"(http:|https:)?(\/\/)?(www\.)?(youtube\.com|youtu\.be)/(watch\?v=)?([a-zA-Z0-9_-]{11})",
    ).unwrap();

    // Arguments pour yt-dlp
    let yt_args = vec![
        Arg::new_with_arg("-f", "bestaudio"),
        Arg::new("-x"),
        Arg::new_with_arg("--audio-format", "flac"),
    ];

    // Initialisation de la sortie audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap_or_else(|err| {
        eprintln!("Error occurred while creating audio stream, do you have an audio output device?: {err}");
        exit(1)
    });

    let sink = Sink::try_new(&stream_handle).unwrap_or_else(|err| {
        eprintln!("Error occurred while creating audio sink: {err}");
        exit(1)
    });

    while !should_exit {
        print!("> ");
        Write::flush(&mut std::io::stdout()).expect("flush failed!");
        
        // Lecture de l'entrée utilisateur
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let mut input = input.trim().split_whitespace();
        let command = input.next();

        match command.unwrap_or("") {
            "play" => {
                let path = input.next().unwrap_or("");
                let mut file = File::open(path);

                if youtube_regex.is_match(path) {
                    println!("YouTube Link Found, using yt-dlp to download audio...");
                    let dir = PathBuf::from(tmp_dir.path());
                    let ytd = YoutubeDL::new(&dir, yt_args.clone(), &path);

                    if let Err(err) = ytd {
                        eprintln!("Error occurred while creating yt-dlp instance: {}", err);
                        continue;
                    }

                    let download = ytd.unwrap().download();
                    if let Err(err) = download {
                        eprintln!("Error occurred while downloading audio: {}", err);
                        continue;
                    }

                    let download = download.unwrap();
                    let filepath = download.output_dir().to_string_lossy().to_string();
                    let output = download.output().split("\n");

                    for line in output {
                        if line.contains("[ExtractAudio] Destination:") {
                            let path = line.split("[ExtractAudio] Destination: ").collect::<Vec<&str>>()[1];
                            let fullpath = format!("{}/{}", filepath, path);
                            file = File::open(fullpath);
                            break;
                        }
                    }
                }

                if let Err(_) = file {
                    eprintln!("File not found");
                    continue;
                }

                let buf = BufReader::new(file.unwrap());
                let source = Decoder::new(buf);
                if let Err(_) = source {
                    eprintln!("Error occurred while decoding audio file");
                    continue;
                }

                if sink.empty() {
                    sink.play();
                    println!("Playing Audio...");
                } else {
                    println!("Adding audio to queue...");
                }
                sink.append(source.unwrap());
            }
            "pause" => {
                sink.pause();
            }
            "resume" => {
                sink.play();
            }
            "p" => {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
            "n" | "next" => {
                if sink.empty() {
                    println!("No more audio to play");
                } else {
                    println!("Playing Next Song...");
                }
                sink.skip_one();
            }
            "s" | "skip" => {
                let skip_amount = input.next().unwrap_or("1").parse::<usize>().unwrap_or(1);
                println!("Skipping {skip_amount} Song(s)...");

                for _ in 0..skip_amount {
                    if sink.empty() {
                        println!("No more audio to play");
                        break;
                    }
                    sink.skip_one();
                }
            }
            "clear" | "c" | "stop" => {
                sink.clear();
                println!("Audio Stopped");
            }
            "exit" => {
                should_exit = true;
            }
            "" => {}
            _ => println!("Unknown command"),
        }

        if command.unwrap_or("") != "" {
            if sink.len() > 0 {
                println!("Position in queue: {}", sink.len());
            } else {
                println!("No audio in queue");
            }
        }
    }

    if should_exit {
        println!("Exiting...");
        sink.stop();
        let _ = tmp_dir.close();
        exit(0);
    }
}


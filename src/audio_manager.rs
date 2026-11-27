// audio_manager.rs
// Música de fondo, pasos en bucle y efectos de daño con rodio

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::{fs::File, io::BufReader, time::Duration};

const FOOTSTEPS: &str = "./audio/sound/step1.wav";
/// El archivo de pasos empieza con medio segundo de silencio
const FOOTSTEPS_LEAD_IN: Duration = Duration::from_millis(450);
const HURT_SOUNDS: [&str; 3] = [
    "./audio/sound/hurt1.wav",
    "./audio/sound/hurt2.wav",
    "./audio/sound/hurt3.wav",
];

pub struct AudioManager {
    /// Mantiene vivo el dispositivo de salida
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    music: Option<Sink>,
    footsteps: Option<Sink>,
}

fn open(path: &str) -> Option<Decoder<BufReader<File>>> {
    let file = File::open(path)
        .map_err(|_| println!("No se encontró el archivo de audio: {path}"))
        .ok()?;
    Decoder::new(BufReader::new(file))
        .map_err(|_| println!("Error decodificando el audio: {path}"))
        .ok()
}

impl AudioManager {
    pub fn new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Self {
                _stream: Some(stream),
                handle: Some(handle),
                music: None,
                footsteps: None,
            },
            Err(_) => {
                println!("No se encontró dispositivo de audio, se desactiva el sonido.");
                Self {
                    _stream: None,
                    handle: None,
                    music: None,
                    footsteps: None,
                }
            }
        }
    }

    fn new_sink(&self) -> Option<Sink> {
        Sink::try_new(self.handle.as_ref()?).ok()
    }

    /// Reproduce música de fondo en bucle, reemplazando la anterior
    pub fn play_music(&mut self, path: &str) {
        if let Some(sink) = self.music.take() {
            sink.stop();
        }
        let (Some(sink), Some(source)) = (self.new_sink(), open(path)) else {
            return;
        };
        sink.append(source.repeat_infinite());
        self.music = Some(sink);
    }

    /// Suena una vez uno de los quejidos, al azar
    pub fn play_hurt(&self) {
        let path = HURT_SOUNDS[rand::random_range(0..HURT_SOUNDS.len())];
        if let (Some(sink), Some(source)) = (self.new_sink(), open(path)) {
            sink.append(source);
            sink.detach();
        }
    }

    /// Los pasos suenan en bucle mientras el jugador camina y se pausan al detenerse
    pub fn set_footsteps(&mut self, walking: bool) {
        if self.footsteps.is_none()
            && walking
            && let (Some(sink), Some(source)) = (self.new_sink(), open(FOOTSTEPS))
        {
            sink.append(source.skip_duration(FOOTSTEPS_LEAD_IN).repeat_infinite());
            sink.set_volume(0.7);
            sink.pause();
            self.footsteps = Some(sink);
        }
        if let Some(sink) = &self.footsteps {
            if walking {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }
}

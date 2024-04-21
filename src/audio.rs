use std::{
	path::PathBuf,
    fs::File,
    io::{BufReader},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread
};
use rodio::{Decoder, OutputStream, OutputStreamHandle, source::Source};
pub struct Sound {
    file_path: PathBuf,
	duration_milliseconds : u64,
	is_stopped : Arc<AtomicBool> 
}

fn _play_sound(
		file_path: String, 
		duration_milliseconds : u64, 
		is_stopped : Arc<AtomicBool>
	) -> Result<OutputStreamHandle, Box<dyn std::error::Error>> {
		
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let file = File::open(file_path)?;
    let source = Decoder::new(BufReader::new(file))?;

    stream_handle.play_raw(source.convert_samples())?;

    let mut elapsed_duration_milliseconds = 0;

    while !is_stopped.load(Ordering::Relaxed) && (elapsed_duration_milliseconds < duration_milliseconds)  {
        thread::sleep(std::time::Duration::from_millis(10));
        elapsed_duration_milliseconds += 10;
    }
    
    Ok(stream_handle)
}

impl Sound {
    // This is an associated function because it does not take `self`.
    pub fn new(file_path: PathBuf, duration_milliseconds : u64) -> Sound {
        Sound {file_path, duration_milliseconds, is_stopped : Arc::new(AtomicBool::new(false))}
    }

	pub fn play(&self) {

		self.is_stopped.store(true, Ordering::Relaxed); 
		
		let file_path_owned: String = self.file_path.to_str().expect("Path is not valid UTF-8").to_string();
		let is_stopped = self.is_stopped.clone();
		let duration_milliseconds = self.duration_milliseconds;

		self.is_stopped.store(false, Ordering::Relaxed); 

		thread::spawn(move || {
			let _ = _play_sound(file_path_owned, duration_milliseconds, is_stopped);
		});
	}

	pub fn stop(&self) {
		self.is_stopped.store(true, Ordering::Relaxed); 
	}
}
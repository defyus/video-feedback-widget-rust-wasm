use crate::{controllers::clips::ClipDetailRequest, helpers::utilities::Utilities};
use actix_files::NamedFile;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{
    fs,
    io::{Error, ErrorKind},
    path::Path,
};

pub struct FFMpegService {}

impl FFMpegService {
    pub async fn get_session_clip_by_id(session_id: String) -> NamedFile {
        let mut temp_directory = dotenv::var("TEMP_DIRECTORY").expect("TEMP_DIRECTORY_NOT_SET");
        temp_directory.push_str(session_id.as_str());

        let output = format!(
            "{}/output-{}.webm",
            temp_directory.clone(),
            session_id.clone()
        );

        NamedFile::open_async(output).await.unwrap()
    }

    pub fn merge_clips(clips: Vec<ClipDetailRequest>, session_id: String) -> Result<String, Error> {
        let mut temp_directory = dotenv::var("TEMP_DIRECTORY").unwrap();
        temp_directory.push_str(session_id.as_str());

        let mut clip_paths: Vec<String> = vec![];

        let mut args: Vec<String> = vec![];
        args.push("-y".to_string());

        //Clip Filter Arg Builder
        let mut filter_complex = String::new();

        let mut clip_idx = 0;

        for clip in clips.iter() {
            let mut path = temp_directory.clone();
            path.push('/');
            path.push_str(clip.id.as_str());
            path.push_str(".webm");

            args.push("-i".to_string());
            args.push(path.clone());

            filter_complex.push_str(format!("[{}:v][{}:a]", clip_idx, clip_idx).as_str());

            clip_paths.push(path.clone());

            clip_idx += 1;
        }

        filter_complex.push_str(format!("concat=n={}:v=1:a=1[outv][outa]", clip_idx).as_str());

        args.push("-filter_complex".to_string());
        args.push(format!("{}", filter_complex));

        args.push("-map".to_string());
        args.push("[outv]".to_string());

        args.push("-map".to_string());
        args.push("[outa]".to_string());

        if clips.len() != clip_paths.len() {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }

        let output = format!(
            "{}/output-{}.webm",
            temp_directory.clone(),
            session_id.clone()
        );

        if Path::new(&output).is_file() {
            fs::remove_file(&output).unwrap();
        }

        //Encoding Settings
        args.push("-c:v".to_string());
        args.push("libvpx-vp9".to_string());
        args.push("-crf".to_string());
        args.push("23".to_string());
        args.push("-c:a".to_string());
        args.push("libopus".to_string());

        //Resource Settings
        args.push("-cpu-used".to_string());
        args.push("8".to_string());
        args.push("-row-mt".to_string());
        args.push("1".to_string());
        args.push("-threads".to_string());
        args.push("8".to_string());
        args.push("-tile-columns".to_string());
        args.push("2".to_string());
        args.push("-frame-parallel".to_string());
        args.push("1".to_string());
        args.push("-auto-alt-ref".to_string());
        args.push("1".to_string());

        args.push(output.to_string());

        let mut command = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("internal_ffmpeg_error");

        if let Some(stderr) = command.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("Progress: {}", line);
                }
            }
        }

        let status = command.wait()?;
        if !status.success() {
            println!("error_success");
        }

        let metadata = std::fs::metadata(output.clone())?;
        if metadata.len() == 0 {
            println!("error_metadata_len");
        }

        Ok(output)
    }

    pub fn create_file(file_path: String, buffer: Vec<u8>) -> String {
        let mut temp_directory = dotenv::var("TEMP_DIRECTORY").expect("TEMP_DIRECTORY_NOT_SET");

        temp_directory.push_str(file_path.as_str());

        if !Path::new(&temp_directory).exists() {
            if let Err(err) = fs::create_dir(temp_directory.clone()) {
                println!("{:?}", err);
            };
        }

        let clip_id = Utilities::rnd_id("clip-");

        temp_directory.push('/');
        temp_directory.push_str(clip_id.as_str());
        temp_directory.push_str(".webm");

        fs::write(temp_directory, buffer).expect("expected_file_write");

        clip_id
    }
}

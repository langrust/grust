mod json {
    //! JSON reading inputs and writing outputs API.
    //!
    //! This API delivers IO functions for JSON inputs/outputs files.
    //!
    //! It is composed of three functions to create a JSON file:
    //! - `begin_json`
    //! - `end_json`
    //! - `append_json`
    //!
    //! The function `read_json` gets the inputs from a JSON file.

    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use serde_json::{self, to_string_pretty, Deserializer};
    use std::fs::OpenOptions;
    use std::io::{self, BufReader, Read, Write};
    use std::path::Path;

    /// Begin a JSON file.
    ///
    /// This function creates a JSON file according to the given path.
    /// The file contains an array of inputs.
    pub fn begin_json<P>(filepath: P)
    where
        P: AsRef<Path>,
    {
        if let Some(p) = filepath.as_ref().parent() {
            std::fs::create_dir_all(p).unwrap()
        };
        std::fs::write(filepath, "").unwrap();
    }

    /// End a JSON file.
    ///
    /// This function ends the JSON file from the given path.
    /// The file contains an array of inputs.
    pub fn end_json<P>(filepath: P)
    where
        P: AsRef<Path>,
    {
        // Open file and append when writing
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filepath)
            .unwrap();
        // Save the JSON structure into the other file.
        writeln!(file, "]").unwrap();
    }

    /// Append an input to a JSON file.
    ///
    /// This function append the given input to the JSON file located at the path.
    pub fn append_json<T, P>(filepath: P, outputs: T)
    where
        T: Serialize,
        P: AsRef<Path>,
    {
        // Open file and append when writing
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(filepath)
            .unwrap();
        // Convert to 'JSON like' String
        let json_string = to_string_pretty(&outputs).unwrap();
        // Save the JSON structure into the other file.
        // Remove last comma
        let size = file.metadata().unwrap().len();
        if size == 0 {
            writeln!(file, "[{}", json_string).unwrap();
        } else {
            writeln!(file, ",{}", json_string).unwrap();
        }
    }

    /// Read a JSON file.
    ///
    /// This function read the JSON file at the given path.
    /// It returns the inputs stored in the file.
    pub fn read_json<T, P>(filepath: P) -> impl Iterator<Item = Result<T, io::Error>>
    where
        T: DeserializeOwned,
        P: AsRef<Path>,
    {
        fn read_skipping_ws(mut reader: impl Read) -> io::Result<u8> {
            loop {
                let mut byte = 0u8;
                reader.read_exact(std::slice::from_mut(&mut byte))?;
                if !byte.is_ascii_whitespace() {
                    return Ok(byte);
                }
            }
        }

        fn invalid_data(msg: &str) -> io::Error {
            io::Error::new(io::ErrorKind::InvalidData, msg)
        }

        fn deserialize_single<T: DeserializeOwned, R: Read>(reader: R) -> io::Result<T> {
            let next_obj = Deserializer::from_reader(reader).into_iter::<T>().next();
            match next_obj {
                Some(result) => result.map_err(Into::into),
                None => Err(invalid_data("premature EOF")),
            }
        }

        fn yield_next_obj<T: DeserializeOwned, R: Read>(
            mut reader: R,
            at_start: &mut bool,
        ) -> io::Result<Option<T>> {
            if !*at_start {
                *at_start = true;
                if read_skipping_ws(&mut reader)? == b'[' {
                    // read the next char to see if the array is empty
                    let peek = read_skipping_ws(&mut reader)?;
                    if peek == b']' {
                        Ok(None)
                    } else {
                        deserialize_single(io::Cursor::new([peek]).chain(reader)).map(Some)
                    }
                } else {
                    Err(invalid_data("`[` not found"))
                }
            } else {
                match read_skipping_ws(&mut reader)? {
                    b',' => deserialize_single(reader).map(Some),
                    b']' => Ok(None),
                    _ => Err(invalid_data("`,` or `]` not found")),
                }
            }
        }

        pub fn iter_json_array<T: DeserializeOwned, R: Read>(
            mut reader: R,
        ) -> impl Iterator<Item = Result<T, io::Error>> {
            let mut at_start = false;
            std::iter::from_fn(move || yield_next_obj(&mut reader, &mut at_start).transpose())
        }

        // Open file for reading
        let file = OpenOptions::new().read(true).open(filepath).unwrap();
        // Create Creates a new BufReader<R> with a default buffer capacity
        // The default is currently 8 KB, but may change in the future
        let reader = BufReader::new(file);

        iter_json_array(reader)
    }
}

use futures::StreamExt;
use interface::{sl_client::SlClient, Input};
use json::*;
use lazy_static::lazy_static;
use std::time::Instant;

// include the `interface` module, which is generated from interface.proto.
pub mod interface {
    tonic::include_proto!("interface");
}

const INPATH: &str = "examples/sl_demo/data/inputs.json";
const OUTPATH: &str = "examples/sl_demo/data/outputs.json";

lazy_static! {
    /// Initial instant.
    static ref INIT : Instant = Instant::now();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect to server
    let mut client = SlClient::connect("http://[::1]:50051").await.unwrap();
    println!("\r\nBidirectional stream (kill client with CTLR+C):");
    // read inputs
    let in_stream = futures::stream::iter(read_json(INPATH)).filter_map(
        move |input: Result<Input, _>| async move {
            match input {
                Ok(input) => {
                    let duration = tokio::time::Duration::from_millis(input.timestamp as u64);
                    let deadline = INIT.clone() + duration;
                    tokio::time::sleep_until(deadline.into()).await;
                    Some(input)
                }
                Err(_) => None,
            }
        },
    );
    // ask for SL service
    let response = client.run_sl(in_stream).await.unwrap();
    // initiate outputs file
    begin_json(OUTPATH);
    // collect all outputs
    let mut resp_stream = response.into_inner();
    let mut counter = 0;
    while let Some(received) = resp_stream.next().await {
        counter += 1;
        let received = received.unwrap();
        println!("\treceived message: `{:?}`", received.message);
        append_json(OUTPATH, received);
        if counter > 10 {
            break;
        }
    }
    end_json(OUTPATH);
    Ok(())
}

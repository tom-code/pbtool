
use std::fs;


use clap::{Parser, Subcommand, ValueEnum};

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::FileDescriptor;
use protobuf::reflect::MessageDescriptor;

#[derive(Debug)]
enum PError {
    Str(String)
}


impl std::fmt::Display for PError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(e) => write!(f, "Error: {}", e.to_string()),
        }
    }
}
impl std::error::Error for PError {}


/// Convert output protobuf binary stream according format specified by FileFormat parameter
fn convert_output(i: &Vec<u8>, format: &Option<FileFormat>) -> Result<Vec<u8>, PError> {
    match format {
        Some(f) if (*f == FileFormat::Binary) => Ok(i.clone()),
        Some(f) if (*f == FileFormat::Base64) => {
            let mut buffer = vec![0u8; i.len()*4];
            match binascii::b64encode(&i, buffer.as_mut()) {
                Ok(res) => return Ok(res.to_vec()),
                Err(err) => return Err(PError::Str(format!("b64 encode failed {:?}", err)))
            };
        }
        _ => {
            let mut buffer =  vec![0u8; i.len()*2];
            match binascii::bin2hex(&i, buffer.as_mut()) {
                Ok(res) => return Ok(res.to_vec()),
                Err(err) => return Err(PError::Str(format!("hex encode failed {:?}", err)))
            }
        }
    }
}

/// Convert input protobuf binary stream according format specified by FileFormat parameter
fn convert_input(i: &String, format: &Option<FileFormat>) -> Result<Vec<u8>, Box<dyn std::error::Error>>  {
    if i.is_empty() {
        return Err(Box::new(PError::Str("nothing to decode".to_string())))
    }

    let input = if i.starts_with("@") {
        fs::read(&i[1..])?
    } else {
        i.as_bytes().to_vec()
    };

    let form = match format {
        Some(f) => f,
        None if i.starts_with("@") => &FileFormat::Binary,
        None  => &FileFormat::Hex,
    };

    match form {
        FileFormat::Binary => return Ok(input),
        FileFormat::Hex => {
            let mut buffer =  vec![0u8; input.len()/2];
            match binascii::hex2bin(&input, buffer.as_mut()) {
                Ok(res) => return Ok(res.to_vec()),
                Err(err) => return Err(Box::new(PError::Str(format!("hex decode failed {:?}", err))))
            }
        }
        FileFormat::Base64 => {
            let mut buffer =  vec![0u8; input.len()];
            match binascii::b64decode(&input, buffer.as_mut()) {
                Ok(res) => return Ok(res.to_vec()),
                Err(err) => return Err(Box::new(PError::Str(format!("b64 decode failed {:?}", err))))
            }
        }
    }
}

/// Get protobuf messageDescriptor from protobuf file for specified probuf message
fn get_message_descriptor(protofile: &String, prototype: &String, include_path: &Option<String>) -> Result<MessageDescriptor, Box<dyn std::error::Error>> {

    let includes = if include_path.is_some() {
        let p: String = include_path.as_ref().unwrap().clone();
        p.split(":").into_iter().map(String::from).collect()
    } else {
        vec!["./".to_string()]
    };

    let file_descriptor_protos_parsed = protobuf_parse::Parser::new()    
    .pure()
    .includes(includes)
    .input(protofile)
    .parse_and_typecheck()?;
    let mut file_descriptor_protos = file_descriptor_protos_parsed.file_descriptors;

    let mut deps = Vec::new();
    for _ in 0..file_descriptor_protos.len()-1 {
        let dep_opt = FileDescriptor::new_dynamic(file_descriptor_protos.remove(0), &deps);
        if let Ok(dep) = dep_opt {
            deps.push(dep)
        } // else error ??
    }

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor = FileDescriptor::new_dynamic(file_descriptor_proto, &deps)?;    

    if let Some(descriptor) = file_descriptor.message_by_full_name(prototype) {
        return Ok(descriptor)
    } else {
        return Err(Box::new(PError::Str(format!("can't find type {}", prototype))))
    }
}


fn encode_internal(protofile: &String, prototype: &String, jsonfile: &String, include_path: &Option<String>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(jsonfile)?;
    let descriptor = get_message_descriptor(protofile, prototype, include_path)?;
    let msg = protobuf_json_mapping::parse_dyn_from_str(&descriptor, &json)?;
    let output_bytes = msg.write_to_bytes_dyn()?;
    return Ok(output_bytes)
}

fn decode_internal(protofile: &String, prototype: &String, indata: &Vec<u8>, include_path: &Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let descriptor = get_message_descriptor(protofile, prototype, include_path)?;
    let pres = descriptor.parse_from_bytes(indata.as_ref())?;
    let jres = protobuf_json_mapping::print_to_string(pres.as_ref())?;
    return Ok(jres)
}

/// encode data specified in json input file into protobuf binary format
fn encode(protofile: &String, prototype: &String, jsonfile: &String, format: &Option<FileFormat>, outfile: &Option<String>, include_path: &Option<String>) {
    let output_bytes = encode_internal(protofile, prototype, jsonfile, include_path).unwrap();

    if format.as_ref().is_some_and(|f| *f == FileFormat::Binary) {
        // do not send binary to stdout
    } else {
        let output_convereted = convert_output(&output_bytes, format).unwrap();
        println!("{}", std::str::from_utf8(output_convereted.as_ref()).unwrap());
    }

    if outfile.is_some() {
        let f = if format.is_none() {
            &Some(FileFormat::Binary)
        } else {
            format
        };
        let output_data_to_file = convert_output(&output_bytes, f).unwrap();
        fs::write(outfile.as_ref().unwrap(), output_data_to_file).unwrap();
    }
}

/// decode protobuf binary data and present them in json format
fn decode(protofile: &String, prototype: &String, data: &String, format: &Option<FileFormat>, out_file: &Option<String>, include_path: &Option<String>) {
    let dataraw = convert_input(data, format).unwrap();

    let jres = decode_internal(protofile, prototype, &dataraw, include_path);
    let json = jres.unwrap();
    if let Some(of) = out_file {
        fs::write(of, json.as_bytes()).unwrap();
    } else {
        println!("{}", json)
    }
}


#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {

    Encode {
        /// Protobuf directory include path
        #[clap(long, short = 'i')]
        include_path: Option<String>,

        /// Output file format
        #[clap(long, short = 'f')]
        file_format: Option<FileFormat>,

        /// Output file name
        #[clap(long, short = 'o')]
        output_file: Option<String>,

        /// name of file with protobuf definition
        protofile: String,
        /// name of protobuf type
        prototype: String,
        /// name of file with json data
        json: String
    },
    Decode {
        /// Protobuf directory include path
        #[clap(long, short = 'i')]
        include_path: Option<String>,

        /// Input format
        #[clap(long, short = 'f')]
        file_format: Option<FileFormat>,

        /// Output file name
        #[clap(long, short = 'o')]
        output_file: Option<String>,

        /// name of file with protobuf definition
        protofile: String,
        /// name of protobuf type
        prototype: String,
        /// protobuf data. name of file when starts with @
        protobuf: String
    },
}
#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum FileFormat {
    Binary,
    Hex,
    Base64,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Encode {
            include_path,
            file_format,
            output_file,
            protofile,
            prototype,
            json } => {
                encode(protofile, prototype, json, file_format, output_file, include_path);
            },
        Commands::Decode {
            include_path,
            file_format,
            output_file,
            protofile,
            prototype,
            protobuf } => {
                decode(protofile, prototype, protobuf, file_format, output_file, include_path)
            }
    }
}

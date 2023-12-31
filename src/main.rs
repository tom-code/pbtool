use std::fs;

use clap::{Parser, Subcommand, ValueEnum};

use protobuf::descriptor::FileDescriptorProto;
use protobuf::reflect::FileDescriptor;
use protobuf::reflect::MessageDescriptor;

/// Convert output protobuf binary stream according format specified by FileFormat parameter
fn convert_output(i: &Vec<u8>, format: &Option<FileFormat>) -> Vec<u8> {
    if format.as_ref().is_some_and(|f| (*f == FileFormat::Binary)) {
        return i.clone()
    }
    if format.as_ref().is_some_and(|f: &FileFormat| (*f == FileFormat::Base64)) {
        let mut buffer = vec![0u8; i.len()*4];
        let res = binascii::b64encode(&i, buffer.as_mut());
        return res.unwrap().to_vec()
    }

    let mut buffer =  vec![0u8; i.len()*2];
    let hex = binascii::bin2hex(&i, buffer.as_mut());
    return hex.unwrap().to_vec()
}

/// Convert input protobuf binary stream according format specified by FileFormat parameter
fn convert_input(i: &String, format: &Option<FileFormat>) -> Vec<u8> {
    if i.len() == 0 {
        return Vec::new()
    }
    if i.starts_with("@") {
        let filename = &i[1..];
        let fdata = fs::read(filename).unwrap();
        if format.is_none() || (*format.as_ref().unwrap() == FileFormat::Binary) {
            return fdata;
        }
        if *format.as_ref().unwrap() == FileFormat::Hex {
            let mut buffer =  vec![0u8; fdata.len()/2];
            let ret = binascii::hex2bin(&fdata, buffer.as_mut());
            return ret.unwrap().to_vec();
        }
        if *format.as_ref().unwrap() == FileFormat::Base64 {
            let mut buffer =  vec![0u8; fdata.len()];
            let ret = binascii::b64decode(&fdata, buffer.as_mut());
            return ret.unwrap().to_vec();
        }
    }
    if format.is_none() || (*format.as_ref().unwrap() == FileFormat::Hex) {
        let mut buffer =  vec![0u8; i.len()/2];
        let ret = binascii::hex2bin(i.as_bytes(), buffer.as_mut());
        return ret.unwrap().to_vec();
    }
    if *format.as_ref().unwrap() == FileFormat::Base64 {
        let mut buffer =  vec![0u8; i.len()];
        let ret = binascii::b64decode(i.as_bytes(), buffer.as_mut());
        return ret.unwrap().to_vec();
    };
    Vec::new()
}



/// Get protobuf messageDescriptor from protobuf file for specified probuf message
fn get_message_descriptor(protofile: &String, prototype: &String, include_path: &Option<String>) -> MessageDescriptor {
    let mut includes = ["./".to_string()].to_vec();
    if include_path.is_some() {
        let p: String = include_path.as_ref().unwrap().clone();
        includes = p.split(":").into_iter().map(String::from).collect();
    }
    let mut file_descriptor_protos = protobuf_parse::Parser::new()    
    .pure()
    .includes(includes)
    .input(protofile)
    .parse_and_typecheck()
    .unwrap()
    .file_descriptors;

    let mut deps = Vec::new();
    for _ in 0..file_descriptor_protos.len()-1 {
        let dep = FileDescriptor::new_dynamic(file_descriptor_protos.remove(0), &deps);
        deps.push(dep.unwrap());
    }

    let file_descriptor_proto: FileDescriptorProto = file_descriptor_protos.pop().unwrap();
    let file_descriptor: FileDescriptor = FileDescriptor::new_dynamic(file_descriptor_proto, &deps).unwrap();

    let descriptor = file_descriptor
        .message_by_full_name(prototype)
        .unwrap();
    return descriptor
}

/// encode data specified in json input file into protobuf binary format
fn encode(protofile: &String, prototype: &String, jsonfile: &String, format: &Option<FileFormat>, outfile: &Option<String>, include_path: &Option<String>) {
    let json = fs::read_to_string(jsonfile).unwrap();

    let m1_descriptor = get_message_descriptor(protofile, prototype, include_path);

    let msg = protobuf_json_mapping::parse_dyn_from_str(&m1_descriptor, &json).unwrap();
    let bytes = msg.write_to_bytes_dyn().unwrap();

    let v = convert_output(&bytes, format);
    println!("{}", std::str::from_utf8(v.as_ref()).unwrap());

    if outfile.is_some() {
        let mut f = format;
        if f.is_none() {
            f = &Some(FileFormat::Binary);
        }
        let vf = convert_output(&bytes, f);
        fs::write(outfile.as_ref().unwrap(), vf).unwrap();
    }
}

/// decode protobuf binary data and present them in json format
fn decode(protofile: &String, prototype: &String, data: &String, format: &Option<FileFormat>, out_file: &Option<String>, include_path: &Option<String>) {
    let descriptor = get_message_descriptor(protofile, prototype, include_path);
    let dataraw = convert_input(data, format);
    let pres = descriptor.parse_from_bytes(dataraw.as_ref());
    let jres = protobuf_json_mapping::print_to_string(pres.unwrap().as_ref());
    if out_file.is_some() {
        fs::write(out_file.as_ref().unwrap(), jres.unwrap().as_bytes()).unwrap();
    } else {
        println!("{}", jres.unwrap())
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
                println!("encode {} {} {}", protofile, prototype, json);
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

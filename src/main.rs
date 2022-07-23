use std::{env, io::Read, path::Path};

use flate2::bufread::ZlibDecoder;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

const IMG_KEY: &[u8; 12] = b"TJPEGImage\r\n";
const SQL_QUERY: &str =
    "SELECT t.标题, s.内容  FROM 标题 as t JOIN 资料库 as s WHERE t.ID = s. fid";
const OUTPUT_FOLDER: &str = "out";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{} <db_file> <out_folder>", &args[0]);
        return;
    }
    let db_path = Path::new(&args[1]);
    let conn = sqlite::open(db_path).unwrap();
    let mut cursor = conn.prepare(SQL_QUERY).unwrap().into_cursor();
    let mut hs = Vec::new();
    while let Some(row) = cursor.next().unwrap() {
        let title = row[0].as_string().unwrap().to_owned();
        if let Some(b) = row[1].as_binary() {
            let data = b.to_vec();
            let out = args[2].to_string();
            let h = tokio::spawn(async move {
                process(data, &title, &out).await;
            });
            hs.push(h);
        }
    }

    for h in hs {
        h.await.unwrap();
    }
}

async fn process(data: Vec<u8>, title: &str, out_folder: &str) {
    let mut buf: Vec<u8> = Vec::new();
    match decompress_zlib(&data, &mut buf) {
        Ok(_) => (),
        Err(e) => println!("decompress error: {}", e),
    };
    extract_images(&buf, title, out_folder).await;
}

fn decompress_zlib(input: &[u8], output: &mut Vec<u8>) -> Result<usize, std::io::Error> {
    let mut decoder = ZlibDecoder::new(input);
    let size = match decoder.read_to_end(output) {
        Ok(s) => s,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(size)
}

/// Extract from `input`. Print error message to std out but not break the process.
async fn extract_images(input: &[u8], title: &str, out_folder: &str) {
    let mut index = 0;
    for i in 0..&input.len() - IMG_KEY.len() + 1 {
        let offset = i + IMG_KEY.len();
        if input[i..offset] == IMG_KEY[..] {
            let e = b"\r";
            for j in offset..input.len() {
                if input[j] == e[0] {
                    let hex_data = &input[offset..j];
                    match hex::decode(hex_data) {
                        Ok(img) => {
                            index += 1;
                            let mut name = String::new();
                            name.push_str(&title.replace(".", "_").replace("?", "_"));
                            name.push_str("_");
                            name.push_str(&index.to_string());
                            let o;
                            if out_folder.is_empty() {
                                o = OUTPUT_FOLDER;
                            } else {
                                o = out_folder;
                            }
                            let p = Path::new(o).join(&name).with_extension("jpg");
                            let prefix = p.parent().unwrap();
                            let _ = fs::create_dir_all(prefix).await;
                            println!("Create file: {}", &p.to_string_lossy());
                            match File::create(&p).await {
                                Ok(mut f) => {
                                    f.write_all(&img).await.unwrap();
                                }
                                Err(e) => println!("Creation failed: {}", e),
                            }
                        }
                        Err(e) => println!("Hex decode failed: {}", e),
                    }
                    break;
                }
            }
        }
    }
}

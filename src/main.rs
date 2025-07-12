use csv::Reader;
use serde::{Deserialize ,Serialize};
use std::error::Error;
use std::fs::File;
use clap::Parser;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct Person {
    name: String,
    age: u32,
    job: String,
}

#[derive(Parser)]
#[command(name = "csv-to-json")]
#[command(about = "csvファイルをJSONに変換するツール")]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}

fn convert_dynamic(input_path: &str, output_path: Option<&str>) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(file);

    let headers = reader.headers()?.clone();
    println!("ヘッダー：{:?}", headers);

    let mut all_rows: Vec<HashMap<String, String>> = Vec::new();

    for result in reader.records() {
        let record = result?;
        let mut row_map = HashMap::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                row_map.insert(header.to_string(), field.to_string());
            }
        }

        all_rows.push(row_map);
    }

    println!("全{}行のデータ:", all_rows.len());
    for (i, row) in all_rows.iter().enumerate() {
        println!("{}行目:{:?}", i + 1, row)
    }

    let json_output = serde_json::to_string_pretty(&all_rows)?;

    match output_path {
        Some(path) => {
            std::fs::write(path, json_output)?;
            println!("JSONファイルを保存しました：{}", path);
        }
        None => {
            println!("JSON出力：");
            println!("{}", json_output);
        }
    }

    Ok(())
}



fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("csv 読み込み開始 ファイル: {}", args.input);

    convert_dynamic(&args.input, args.output.as_deref())?;

    Ok(())
}

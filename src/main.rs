use csv::Reader;
use std::fs::File;
use clap::Parser;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

// cargo run -- --input sample.csv --output result.json --stats

#[derive(Parser)]
#[command(name = "csv-to-json")]
#[command(about = "csvファイルをJSONに変換するツール")]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long, help = "統計情報を表示する")]
    stats: bool,
}

#[derive(Error, Debug)]
enum ConversionError {
    #[error("🚨 ファイルが見つかりません: {path}\n💡 解決方法: ファイルパスを確認してください")]
    FileNotFound { path: String },
    
    #[error("🚨 ファイル読み込みエラー: {path}\n💡 解決方法: ファイルの権限を確認してください")]
    FileReadError { path: String },
    
    #[error("🚨 CSVファイルの形式が不正です\n💡 解決方法: CSV形式を確認してください（ヘッダー行、区切り文字など）")]
    CsvParseError,
    
    #[error("🚨 CSVデータの読み込みでエラーが発生しました: {line}\n💡 解決方法: {line}行目のデータを確認してください")]
    CsvRecordError { line: usize },
    
    #[error("🚨 JSON変換でエラーが発生しました\n💡 解決方法: CSVデータに特殊文字が含まれている可能性があります")]
    JsonConversionError,
    
    #[error("🚨 ファイル書き込みエラー: {path}\n💡 解決方法: 書き込み権限とディスクの空き容量を確認してください")]
    FileWriteError { path: String },
}

#[derive(Debug)]
struct CsvStats {
    total_rows: usize,
    total_columns: usize,
    empty_cells: usize,
    column_unique_counts: HashMap<String, usize>
}

impl CsvStats {
    fn new() -> Self {
        CsvStats {
            total_rows: 0,
            total_columns: 0,
            empty_cells: 0,
            column_unique_counts: HashMap::new(),
        }
    }

    fn display(&self) {
        println!("データ統計情報");
        println!("---------------------------");
        println!("総行数：{}行", self.total_rows);
        println!("列数：{}行", self.total_columns);
        println!("空のセル：{}個", self.empty_cells);
        println!("ユニークな値の数：");

        for (column, count) in &self.column_unique_counts {
            println!("   - {}: {}個", column, count);
        }
        println!("---------------------------");
    }
}

fn calculate_stats(data: &[HashMap<String, String>], headers: &csv::StringRecord) -> CsvStats {
    let mut stats = CsvStats::new();

    stats.total_rows =data.len();
    stats.total_columns = headers.len();

let mut column_unique_values: HashMap<String, HashSet<String>> = HashMap::new();

for header in headers.iter() {
    column_unique_values.insert(header.to_string(), HashSet::new());
}


    for row in data {
        for (column, value) in row {

            if value.trim().is_empty() {
                stats.empty_cells += 1;
            }

            if let Some(unique_set) = column_unique_values.get_mut(column) {
                unique_set.insert(value.clone());
            }
        }
    }

    for (column, unique_set) in column_unique_values {
        stats.column_unique_counts.insert(column, unique_set.len());
    }

    stats
}

fn convert_dynamic(input_path: &str, output_path: Option<&str>, show_stats: bool) -> Result<(), ConversionError> {
    // let file = File::open(input_path)?;

    let file = File::open(input_path).map_err(|e| {
        match e.kind() {
            std::io::ErrorKind::NotFound => ConversionError::FileNotFound { path: input_path.to_string(),
            },
            _ => ConversionError::FileReadError { 
            path: input_path.to_string(),
            },
        }
    })?;

    let mut reader = Reader::from_reader(file);

    let headers = reader.headers().map_err(|_| ConversionError::CsvParseError)?.clone();
    println!("ヘッダー読み込み完了：{:?}", headers);

    let mut all_rows: Vec<HashMap<String, String>> = Vec::new();

    for (line_num, result) in reader.records().enumerate() {
        let record = result.map_err(|_| ConversionError::CsvRecordError { 
            line: line_num + 2
        })?;
        let mut row_map = HashMap::new();

        for (i, field) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                row_map.insert(header.to_string(), field.to_string());
            }
        }

        all_rows.push(row_map);
    }

    println!("全{}行のデータ読み込み完了:", all_rows.len());
    for (i, row) in all_rows.iter().enumerate().take(3) {
        println!("{}行目:{:?}", i + 1, row)
    }
    if all_rows.len() > 3 {
        println!("...(他{}行)", all_rows.len() -3);
    }

    if show_stats {
        let stats = calculate_stats(&all_rows, &headers);
        stats.display();
    }

    let json_output = serde_json::to_string_pretty(&all_rows).map_err(|_| ConversionError::JsonConversionError)?;

    match output_path {
        Some(path) => {
            std::fs::write(path, json_output).map_err(|_| ConversionError::FileWriteError { path: path.to_string(), })?;
            println!("JSONファイルを保存しました：{}", path);
        }
        None => {
            println!("JSON出力：");
            println!("{}", json_output);
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    println!("csv 読み込み開始 ファイル: {}", args.input);
    println!("─────────────────────────────────────");

    if let Err(e) = convert_dynamic(&args.input, args.output.as_deref(), args.stats) {
        eprintln!("\n{}", e);
        std::process::exit(1);
    };

    println!("─────────────────────────────────────");
    println!("🎉 変換完了！お疲れ様でした〜");
}

use parser::{parse_schema, parse_sysctl};
use std::fs::File;
use std::io::{self, Read};
use std::{env, path::Path};
use validation::validate_by_schema;

mod parser;
mod types;
mod validation;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let input_file_path = &args[1];
    let use_validation = match &args.get(2) {
        Some(v) => {
            if *v == "--validate" || *v == "-v" {
                true
            } else {
                false
            }
        }
        None => false,
    };

    let input_str = read_file(&input_file_path).expect("ファイルの読み込みに失敗しました。");
    let parse_sysctl_result = parse_sysctl(&input_str);
    if parse_sysctl_result.is_err() {
        println!("文法に誤りがあります。");
        std::process::exit(1);
    }
    let sysctl_data = parse_sysctl_result.unwrap().1;

    let schema_file_path = format!("{}.schema", input_file_path);
    if use_validation && Path::new(&schema_file_path).exists() {
        let schema_str =
            read_file(&schema_file_path).expect("スキーマファイルの読み込みに失敗しました。");

        let parse_schema_result = parse_schema(&schema_str);
        if parse_schema_result.is_err() {
            println!("スキーマファイルの文法に誤りがあります");
            std::process::exit(1);
        }
        let schema = parse_schema_result.unwrap().1;

        if let Err(validation_errors) = validate_by_schema(&sysctl_data, &schema) {
            println!("スキーマエラーがありました。");
            for error in validation_errors {
                match error {
                    types::ValidationError::MissingKey(key) => {
                        println!("必要なキーである'{}'が存在しません", key);
                    }
                    types::ValidationError::UnknownKey(key) => {
                        println!("定義されていない'{}'が存在しており、これは不要です", key)
                    }
                    types::ValidationError::WrongType {
                        key_name,
                        expect,
                        actual,
                    } => {
                        println!(
                            "'{}'の型が間違っています。{}が必要ですが、{}の形式になっています。",
                            key_name, expect, actual
                        )
                    }
                    types::ValidationError::TooLongLine(key) => {
                        println!("'{}'の値の行長が最大である4096を超えています。", key);
                    }
                }
            }
            std::process::exit(1);
        } else {
            println!(
                "スキーマエラーはありませんでした。読み込んだデータをRust形式で出力します。{:#?}",
                &sysctl_data
            );
        }
    } else {
        println!(
            "読み込んだデータをRust形式で出力します。{:#?}",
            &sysctl_data
        );
    }

    Ok(())
}

fn read_file(file_path: &str) -> io::Result<String> {
    let mut buffer = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

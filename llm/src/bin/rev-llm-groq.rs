use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::SystemTime;

use filetime::FileTime;
use reqwest::blocking::Client;
use serde_json::{json, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let draft_file = format!("{}.draft", output_file);

    let api_key = env::var("GROQ_API_KEY")
        .expect("GROQ_API_KEY environment variable must be set");
    let client = Client::new();

    let mut input_content = String::new();
    File::open(input_file)?.read_to_string(&mut input_content)?;

    let prompt: String;
    let output_exists = Path::new(output_file).exists();
    let output_empty = if output_exists {
        let metadata = fs::metadata(output_file)?;
        metadata.len() == 0
    } else {
        true
    };

    if output_exists && !output_empty {
        let mut output_content = String::new();
        File::open(output_file)?.read_to_string(&mut output_content)?;

        prompt = format!(
            "Please verify that the implementation below (enclosed into <result-specimen></result-specimen>) is accurately described by the specification (enclosed into <result-specification></result-specification>) as much as possible. If it does - then simply output the content of the result-specification verbatim. If you find that there are imperfections in how result-specification describes the specimen, then incrementally improve it and output the full result, with your improvements. Do not delimit the result with anything, output it verbatim.\n\n<result-specimen>\n{}\n</result-specimen>\n\n<result-specification>\n{}\n</result-specification>",
            input_content.trim(),
            output_content.trim()
        );
    } else {
        prompt = format!(
            "Please produce a detailed specification which will allow to recreate the implementation below from first principles:\n\n{}",
            input_content.trim()
        );
    }

    eprintln!("Calling Groq API for initial response...");
    let response = call_groq(&client, &api_key, &prompt, 0.7, 16384)?;
    let draft_path = Path::new(&draft_file);
    let mut draft_file_handle = File::create(draft_path)?;
    draft_file_handle.write_all(response.trim().as_bytes())?;

    if output_exists && !output_empty {
        eprintln!("Evaluating specification improvements...");
        let evaluation_prompt = format!(
            "Please CAREFULLY evaluate the below specimen (enclosed into <result-specimen></result-specimen>), and two outputs corresponding to this description, first one enclosed into \"<first-specification></first-specification>\" and the second enclosed into \"<second-specification></second-specification>\", and evaluate which of the two is more precise and correct in describing the specimen. Then, if the first result is better, output the phrase 'First specification is better.', if the second description is better, output the phrase 'The second spec is better.'. Output only one of the two phrases, and nothing else\n\n<result-specimen>\n{}\n</result-specimen>\n\n<first-specification>\n{}\n</first-specification>\n\n<second-specification>\n{}</second-specification>",
            input_content.trim(),
            output_file_content(output_file)?.trim(),
            response.trim()
        );

        let eval_response = call_groq(&client, &api_key, &evaluation_prompt, 0.1, 100)?;

        match eval_response.trim() {
            "The second spec is better." => {
                let mut final_file = File::create(output_file)?;
                final_file.write_all(response.trim().as_bytes())?;
                eprintln!("Updated {}", output_file);
            }
            "First specification is better." => {
                let rejected_path = format!("{}.rej", output_file);
                fs::rename(&draft_file, &rejected_path)?;
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs() as i64;
                filetime::set_file_mtime(output_file, FileTime::from_unix_time(now, 0))?;
                eprintln!("Kept original specification, rejected draft saved to {}", rejected_path);
            }
            _ => {
                eprintln!("Unexpected evaluation response: {}", eval_response.trim());
                std::process::exit(1);
            }
        }
    } else {
        let mut final_file = File::create(output_file)?;
        final_file.write_all(response.trim().as_bytes())?;
        eprintln!("Created {}", output_file);
    }

    Ok(())
}

fn call_groq(
    client: &Client,
    api_key: &str,
    prompt: &str,
    temperature: f64,
    max_tokens: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let body = json!({
        "model": "moonshotai/kimi-k2-instruct",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": temperature,
        "max_tokens": max_tokens
    });

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    if !response.status().is_success() {
        eprintln!("API call failed: {}", response.status());
        eprintln!("Response: {}", response.text()?);
        std::process::exit(1);
    }

    let response_json: Value = response.json()?;
    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Failed to extract content from response")?;

    Ok(content.to_string())
}

fn output_file_content(path: &str) -> io::Result<String> {
    let mut content = String::new();
    if Path::new(path).exists() {
        File::open(path)?.read_to_string(&mut content)?;
    }
    Ok(content)
}


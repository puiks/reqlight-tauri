use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use reqlight_lib::models::collection::RequestCollection;
use reqlight_lib::models::state::AppState;
use reqlight_lib::models::{KeyValuePair, SavedRequest};
use reqlight_lib::services::{assertion, http_client, interpolator, junit};

/// Reqlight CLI — Run API test collections from the command line.
#[derive(Parser)]
#[command(name = "reqlight", version, about)]
struct Cli {
    /// Path to data.json or a single collection JSON file
    #[arg(short, long)]
    file: PathBuf,

    /// Collection name to run (required when --file points to data.json with multiple collections)
    #[arg(short, long)]
    collection: Option<String>,

    /// Environment name to use for variable interpolation
    #[arg(short, long)]
    env: Option<String>,

    /// CSV file for data-driven testing (columns become environment variables)
    #[arg(short, long)]
    data: Option<PathBuf>,

    /// Output JUnit XML report to this file
    #[arg(short, long)]
    junit: Option<PathBuf>,

    /// Stop on first failure
    #[arg(long, default_value_t = false)]
    fail_fast: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let (collection, env_vars) = match load_inputs(&cli) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let data_rows = match load_data_file(&cli.data) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error loading data file: {e}");
            return ExitCode::FAILURE;
        }
    };

    let iterations = if data_rows.is_empty() {
        vec![vec![]] // single iteration with no extra vars
    } else {
        data_rows
    };

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client");

    let mut all_results: Vec<junit::TestResult> = Vec::new();
    let mut total_pass = 0usize;
    let mut total_fail = 0usize;
    let mut exit_early = false;

    for (iter_idx, extra_vars) in iterations.iter().enumerate() {
        let iter_label = if iterations.len() > 1 {
            format!(" [row {}]", iter_idx + 1)
        } else {
            String::new()
        };

        for request in &collection.requests {
            let merged_vars = merge_vars(&env_vars, extra_vars);
            let result = run_request(&client, request, &merged_vars).await;

            let status_icon = if result.passed { "✓" } else { "✗" };
            let url_display = request.url.to_string();
            let status_code = result.error_message.as_deref().unwrap_or(&url_display);
            let time_str = format!("{:.0}ms", result.elapsed_secs * 1000.0);

            if result.passed {
                println!(
                    "  {status_icon} {} {}{iter_label} [{time_str}]",
                    request.method.as_str(),
                    request.name,
                );
                total_pass += 1;
            } else {
                eprintln!(
                    "  {status_icon} {} {}{iter_label} — {status_code} [{time_str}]",
                    request.method.as_str(),
                    request.name,
                );
                for ar in &result.assertion_results {
                    if !ar.passed {
                        eprintln!("    ✗ {}", ar.message);
                    }
                }
                total_fail += 1;
            }

            all_results.push(result);

            if !all_results.last().unwrap().passed && cli.fail_fast {
                exit_early = true;
                break;
            }
        }
        if exit_early {
            break;
        }
    }

    println!("\n{total_pass} passed, {total_fail} failed");

    // Write JUnit report if requested
    if let Some(ref junit_path) = cli.junit {
        let xml = junit::generate_junit_xml(&collection.name, &all_results);
        if let Err(e) = std::fs::write(junit_path, &xml) {
            eprintln!("Failed to write JUnit report: {e}");
        } else {
            println!("JUnit report written to {}", junit_path.display());
        }
    }

    if total_fail > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn load_inputs(cli: &Cli) -> Result<(RequestCollection, Vec<KeyValuePair>), String> {
    let content = std::fs::read_to_string(&cli.file)
        .map_err(|e| format!("Cannot read {}: {e}", cli.file.display()))?;

    // Try parsing as AppState (data.json) first
    if let Ok(state) = serde_json::from_str::<AppState>(&content) {
        let collection = match &cli.collection {
            Some(name) => state
                .collections
                .into_iter()
                .find(|c| c.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| format!("Collection '{name}' not found"))?,
            None => {
                if state.collections.len() == 1 {
                    state.collections.into_iter().next().unwrap()
                } else {
                    let names: Vec<_> = state.collections.iter().map(|c| c.name.as_str()).collect();
                    return Err(format!(
                        "Multiple collections found. Use --collection to specify one: {}",
                        names.join(", ")
                    ));
                }
            }
        };

        let env_vars = match &cli.env {
            Some(env_name) => state
                .environments
                .into_iter()
                .find(|e| e.name.eq_ignore_ascii_case(env_name))
                .map(|e| e.variables)
                .unwrap_or_default(),
            None => vec![],
        };

        return Ok((collection, env_vars));
    }

    // Try parsing as a single RequestCollection
    if let Ok(collection) = serde_json::from_str::<RequestCollection>(&content) {
        return Ok((collection, vec![]));
    }

    Err("File is neither a valid data.json nor a collection JSON".to_string())
}

fn load_data_file(path: &Option<PathBuf>) -> Result<Vec<Vec<KeyValuePair>>, String> {
    let path = match path {
        Some(p) => p,
        None => return Ok(vec![]),
    };

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "csv" => load_csv(path),
        "json" => load_json_data(path),
        _ => Err(format!(
            "Unsupported data file format: .{ext} (use .csv or .json)"
        )),
    }
}

fn load_csv(path: &PathBuf) -> Result<Vec<Vec<KeyValuePair>>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| format!("CSV read error: {e}"))?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| format!("CSV header error: {e}"))?
        .iter()
        .map(String::from)
        .collect();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| format!("CSV row error: {e}"))?;
        let vars: Vec<KeyValuePair> = headers
            .iter()
            .zip(record.iter())
            .map(|(key, value)| KeyValuePair {
                id: uuid::Uuid::new_v4(),
                key: key.clone(),
                value: value.to_string(),
                is_enabled: true,
                is_secret: false,
            })
            .collect();
        rows.push(vars);
    }
    Ok(rows)
}

fn load_json_data(path: &PathBuf) -> Result<Vec<Vec<KeyValuePair>>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("JSON read error: {e}"))?;
    let array: Vec<HashMap<String, serde_json::Value>> =
        serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {e}"))?;

    let rows = array
        .into_iter()
        .map(|obj| {
            obj.into_iter()
                .map(|(key, value)| KeyValuePair {
                    id: uuid::Uuid::new_v4(),
                    key,
                    value: match value {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    },
                    is_enabled: true,
                    is_secret: false,
                })
                .collect()
        })
        .collect();
    Ok(rows)
}

fn merge_vars(base: &[KeyValuePair], extra: &[KeyValuePair]) -> Vec<KeyValuePair> {
    let mut merged = base.to_vec();
    for var in extra {
        if let Some(existing) = merged.iter_mut().find(|v| v.key == var.key) {
            existing.value.clone_from(&var.value);
        } else {
            merged.push(var.clone());
        }
    }
    merged
}

async fn run_request(
    client: &reqwest::Client,
    request: &SavedRequest,
    variables: &[KeyValuePair],
) -> junit::TestResult {
    // Warn about unresolved variables
    let unmatched = interpolator::find_unmatched(&request.url, variables);
    if !unmatched.is_empty() {
        let vars: Vec<_> = unmatched.iter().map(|v| format!("{{{{{v}}}}}")).collect();
        eprintln!("    ⚠ Unresolved variables: {}", vars.join(", "));
    }

    let (url, headers, params, body) = interpolator::interpolate_request(
        &request.url,
        &request.headers,
        &request.query_params,
        &request.body,
        variables,
    );
    let auth = interpolator::interpolate_auth(&request.auth, variables);

    match http_client::execute(
        client,
        &request.method,
        &url,
        &headers,
        &params,
        &body,
        &auth,
        request.timeout_secs,
        None,
        None,
    )
    .await
    {
        Ok(response) => {
            let assertion_results = assertion::evaluate(&request.assertions, &response);
            let passed = if assertion_results.is_empty() {
                response.status_code >= 200 && response.status_code < 300
            } else {
                assertion_results.iter().all(|r| r.passed)
            };

            junit::TestResult {
                name: request.name.clone(),
                method: request.method.as_str().to_string(),
                url,
                elapsed_secs: response.elapsed_time / 1000.0,
                passed,
                error_message: None,
                assertion_results,
            }
        }
        Err(ref e) => junit::TestResult {
            name: request.name.clone(),
            method: request.method.as_str().to_string(),
            url,
            elapsed_secs: 0.0,
            passed: false,
            error_message: Some(e.to_string()),
            assertion_results: vec![],
        },
    }
}

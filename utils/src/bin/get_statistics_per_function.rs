use anyhow::{Context, Result};
use clap::Parser;
use tabled::{
    settings::{object::Columns, Color, Style},
    Table,
};
use utils::{
    cloudwatch_logs::get_lambda_statistics,
    tfstate_parser::{get_lambda_function_names, TerraformState},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Path to terraform tfstate
    tfstate_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let tf_state = tokio::fs::read_to_string(args.tfstate_path)
        .await
        .context("could not read TF state file")?;
    let tf_state: TerraformState =
        serde_json::from_str(&tf_state).context("could not deserialize TF state")?;

    let lambda_function_names: Vec<_> = get_lambda_function_names(&tf_state);

    let mut query_results = get_lambda_statistics(&lambda_function_names).await?;
    #[allow(clippy::cast_possible_truncation)]
    query_results.sort_by_key(|res| (res.function_name.clone(), res.memory_set as i64));

    let mut table = Table::new(query_results);
    table
        .with(Style::psql())
        .modify(Columns::single(0), Color::FG_RED)
        .modify(Columns::new(1..), Color::FG_GREEN);
    println!("Results:\n{table}");

    Ok(())
}

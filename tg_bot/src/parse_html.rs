use std::fs;

mod html_parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    parse_html("data/ai_product.html", "data/ai_product_parsed.json")?;
    parse_html("data/ai.html", "data/ai_parsed.json")?;
    Ok(())
}

fn parse_html(html_path: &str, output_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the HTML file
    let html_content = fs::read_to_string(html_path)?;
    
    // Parse the HTML content
    let program = html_parser::parse_master_program_html(&html_content)?;
    
    // Convert to JSON
    let json_output = serde_json::to_string_pretty(&program)?;
    
    // Save to file
    fs::write(output_json, &json_output)?;
    
    println!("\nParsed data saved to: {}", output_json);
    
    Ok(())
}
use config_lib::parsers::detect_format;

fn main() -> config_lib::Result<()> {
    let content = std::fs::read_to_string("test.ini").unwrap();

    println!("=== Content ===");
    println!("{}", content);

    println!("\n=== Detection Process ===");

    // Check each detection function manually
    println!(
        "Contains NOML features: {}",
        contains_noml_features(&content)
    );
    println!(
        "Contains Properties features: {}",
        contains_properties_features(&content)
    );
    println!("Contains INI features: {}", contains_ini_features(&content));
    println!(
        "Contains TOML features: {}",
        contains_toml_features(&content)
    );

    println!("\nFinal detection: {}", detect_format(&content));

    Ok(())
}

// Copy the detection functions locally for testing
fn contains_noml_features(content: &str) -> bool {
    content.contains("env(")
        || content.contains("include ")
        || content.contains("${")
        || content.contains("@size(")
        || content.contains("@duration(")
        || content.contains("@url(")
        || content.contains("@ip(")
}

fn contains_properties_features(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('!') {
            return true;
        }
        if trimmed.contains("\\n") || trimmed.contains("\\t") || trimmed.contains("\\u") {
            return true;
        }
        if trimmed.contains(':') && !trimmed.contains('=') && !trimmed.starts_with('#') {
            return true;
        }
        false
    })
}

fn contains_ini_features(content: &str) -> bool {
    let mut has_section = false;
    let mut has_ini_comment = false;
    let mut has_key_value_in_section = false;
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.contains('=') {
            let section_content = &trimmed[1..trimmed.len() - 1];
            if !section_content.contains('[') && !section_content.contains(']') {
                has_section = true;
                in_section = true;
                continue;
            }
        }

        if trimmed.starts_with(';') {
            has_ini_comment = true;
        }

        if in_section
            && (trimmed.contains('=') || trimmed.contains(':'))
            && !trimmed.starts_with('#')
            && !trimmed.starts_with(';')
        {
            has_key_value_in_section = true;
        }
    }

    println!(
        "INI detection details: has_section={}, has_ini_comment={}, has_key_value_in_section={}",
        has_section, has_ini_comment, has_key_value_in_section
    );

    has_section && has_key_value_in_section || has_ini_comment
}

fn contains_toml_features(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.contains('=') {
            return true;
        }
        if trimmed.contains("T") && trimmed.contains("Z") {
            return true;
        }
        false
    })
}

use config_lib::Config;

fn main() {
    let content = r#"
        # Basic configuration
        app_name = "test-app"
        port = 8080
        debug = true
        version = 1.0
        
        # Section
        [database]
        host = "localhost"
        port = 5432
        
        # Arrays
        servers = alpha beta gamma
        ports = 8001 8002 8003
    "#;

    match Config::from_string(content, Some("conf")) {
        Ok(config) => {
            println!("Success! Parsed config");

            println!("Root level keys:");

            // Check for servers in root
            match config.get("servers") {
                Some(value) => println!("Found servers in root: {value:?}"),
                None => println!("servers not found in root"),
            }

            // Check for servers in database section
            match config.get("database.servers") {
                Some(value) => println!("Found servers in database: {value:?}"),
                None => println!("servers not found in database section"),
            }

            // Check for ports in database section
            match config.get("database.ports") {
                Some(value) => println!("Found ports in database: {value:?}"),
                None => println!("ports not found in database section"),
            }
        }
        Err(e) => {
            println!("Error: {e:?}");
        }
    }
}

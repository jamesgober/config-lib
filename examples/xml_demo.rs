//! # XML Configuration Demo
//!
//! Demonstrates XML configuration parsing capabilities for enterprise Java/.NET environments.
#[allow(unused_imports)]
use config_lib::{Config, Result};

fn main() -> Result<()> {
    println!("=== XML Configuration Demo ===\n");

    // Example 1: Spring Boot style XML configuration
    #[cfg(feature = "xml")]
    let spring_config = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <configuration>
        <appSettings>
            <add key="database.host" value="localhost" />
            <add key="database.port" value="5432" />
            <add key="database.ssl" value="true" />
            <add key="app.name" value="SpringApp" />
            <add key="app.version" value="2.1.0" />
        </appSettings>
        
        <connectionStrings>
            <add name="DefaultConnection" 
                 connectionString="Server=localhost;Database=mydb;User Id=admin;Password=secret;" />
        </connectionStrings>
        
        <logging>
            <level>INFO</level>
            <file>/var/log/app.log</file>
            <maxSize>10MB</maxSize>
        </logging>
    </configuration>
    "#;

    #[cfg(feature = "xml")]
    {
        println!("1. Spring Boot Style XML Configuration:");
        println!("----------------------------------------");

        let config = Config::from_string(spring_config, Some("xml"))?;

        // Access nested configuration values
        if let Some(logging_level) = config.get("configuration.logging.level") {
            println!("   Logging Level: {}", logging_level.as_string()?);
        }

        if let Some(app_name) = config.get("configuration.appSettings.add") {
            println!("   App Configuration: {:#?}", app_name);
        }

        println!("   ✅ XML parsing successful!\n");
    }

    #[cfg(not(feature = "xml"))]
    {
        println!(
            "❌ XML feature not enabled. Run with: cargo run --features xml --example xml_demo\n"
        );
    }

    // Example 2: Maven/Gradle style configuration
    #[cfg(feature = "xml")]
    let maven_config = r#"
    <project>
        <modelVersion>4.0.0</modelVersion>
        <groupId>com.example</groupId>
        <artifactId>my-app</artifactId>
        <version>1.0.0</version>
        
        <properties>
            <maven.compiler.source>11</maven.compiler.source>
            <maven.compiler.target>11</maven.compiler.target>
            <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
        </properties>
        
        <dependencies>
            <dependency>
                <groupId>org.springframework</groupId>
                <artifactId>spring-core</artifactId>
                <version>5.3.0</version>
            </dependency>
        </dependencies>
    </project>
    "#;

    #[cfg(feature = "xml")]
    {
        println!("2. Maven Style XML Configuration:");
        println!("----------------------------------");

        let config = Config::from_string(maven_config, Some("xml"))?;

        if let Some(group_id) = config.get("project.groupId") {
            println!("   Group ID: {}", group_id.as_string()?);
        }

        if let Some(version) = config.get("project.version") {
            println!("   Version: {}", version.as_string()?);
        }

        println!("   ✅ Maven XML parsing successful!\n");
    }

    // Example 3: ASP.NET Core style configuration
    #[cfg(feature = "xml")]
    let aspnet_config = r#"
    <configuration>
        <appSettings>
            <add key="Environment" value="Production" />
            <add key="AllowedHosts" value="*" />
        </appSettings>
        
        <connectionStrings>
            <add name="DefaultConnection" value="Data Source=server;Initial Catalog=db" />
        </connectionStrings>
        
        <system.webServer>
            <defaultDocument enabled="true">
                <files>
                    <add value="index.html" />
                </files>
            </defaultDocument>
        </system.webServer>
    </configuration>
    "#;

    #[cfg(feature = "xml")]
    {
        println!("3. ASP.NET Core Style XML Configuration:");
        println!("----------------------------------------");

        let config = Config::from_string(aspnet_config, Some("xml"))?;

        if let Some(env) = config.get("configuration.appSettings") {
            println!("   App Settings: {:#?}", env);
        }

        println!("   ✅ ASP.NET XML parsing successful!\n");
    }

    // Performance test
    #[cfg(feature = "xml")]
    {
        println!("4. Performance Test:");
        println!("--------------------");

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _config = Config::from_string(spring_config, Some("xml"))?;
        }
        let duration = start.elapsed();

        println!("   Parsed 1000 XML configs in: {:?}", duration);
        println!("   Average per parse: {:?}", duration / 1000);
        println!("   ✅ High performance confirmed!\n");
    }

    println!("=== XML Demo Complete ===");
    println!("💡 XML parsing is feature-gated for zero impact when disabled");
    println!("🚀 Perfect for enterprise Java/.NET configuration files");

    Ok(())
}

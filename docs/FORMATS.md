<h1 id="top" align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>config-lib</b><br>
    <sub><sup>VALID FORMATS</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./README.md" title="Documentation"><b>DOCS</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="./API.md" title="Formats"><b>API</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>FORMATS</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="./GUIDELINES.md" title="Developer Guidelines"><b>GUIDELINES</b></a>
    </sup>
</div>

<br>

## Table of Contents

- **[Conf](#conf)**
- **[INI](#ini)**
- **[Properties](#properties)**
- **[JSON](#json)**
- **[NOML](#noml)**
- **[TOML](#toml)**
- **[XML](#xml)**
- **[HCL](#hcl)**

<hr>
<br>

<h2 id="conf">Conf</h2>
<p>
    <strong>Conf</strong> is a simple, human-readable configuration format commonly used in Unix/Linux systems. It supports key-value pairs, sections, comments, and nested configurations using a clean, minimal syntax that's easy to read and maintain.
</p>

**Key Features:**
- Simple key-value pairs
- Section support with `[section]` headers
- Comments with `#` prefix
- Multi-line values
- Environment variable substitution
- Nested configurations

**Basic Configuration:**
```conf
# Application Configuration
app.name = "MyApplication"
app.version = "1.0.0"
app.debug = true
app.environment = "development"

# Server Settings
server.host = "localhost"
server.port = 8080
server.timeout = 30
server.workers = 4

# Database Configuration
database.host = "localhost"
database.port = 5432
database.name = "myapp_db"
database.username = "postgres"
database.password = "secret123"
database.pool_size = 10
database.ssl_mode = "prefer"

# Logging Configuration
logging.level = "info"
logging.file = "/var/log/app.log"
logging.max_size = "100MB"
logging.rotate = true
logging.retention_days = 30
```

**Section-Based Configuration:**
```conf
# Global settings
app_name = "Enterprise Application"
version = "2.1.0"

[server]
host = "0.0.0.0"
port = 8443
ssl_enabled = true
ssl_cert = "/etc/ssl/certs/app.crt"
ssl_key = "/etc/ssl/private/app.key"
max_connections = 1000
timeout = 60

[database]
driver = "postgresql"
host = "db.example.com"
port = 5432
name = "production_db"
username = "app_user"
password = "${DB_PASSWORD}"
pool_size = 20
connection_timeout = 30
ssl_mode = "require"

[cache]
enabled = true
engine = "redis"
host = "cache.example.com"
port = 6379
ttl = 3600
max_memory = "256MB"

[logging]
level = "warn"
format = "json"
output = "file"
file = "/var/log/app/application.log"
max_size = "500MB"
max_files = 10
compress = true

[features]
analytics = false
monitoring = true
api_versioning = true
rate_limiting = true
debug_mode = false
```

**Advanced Configuration with Arrays and Multi-line Values:**
```conf
# Multi-line values
description = "This is a multi-line description\n" +
              "that spans multiple lines and\n" +
              "provides detailed information."

# Array-like configurations
allowed_hosts = "localhost,127.0.0.1,example.com"
allowed_ips = "192.168.1.0/24,10.0.0.0/8"

# Complex nested configuration
auth.providers.oauth.google.client_id = "google-client-id"
auth.providers.oauth.google.client_secret = "${GOOGLE_CLIENT_SECRET}"
auth.providers.oauth.google.redirect_uri = "https://app.example.com/auth/callback"

auth.providers.oauth.github.client_id = "github-client-id"
auth.providers.oauth.github.client_secret = "${GITHUB_CLIENT_SECRET}"
auth.providers.oauth.github.scope = "user:email"

# API configuration
api.rate_limit.requests_per_minute = 1000
api.rate_limit.burst_limit = 100
api.cors.allowed_origins = "*"
api.cors.allowed_methods = "GET,POST,PUT,DELETE"
api.cors.allowed_headers = "Content-Type,Authorization"
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="ini">INI</h2>
<p>
    <strong>INI</strong> (Initialization) files are a traditional configuration format widely used in Windows and cross-platform applications. They organize settings into sections with key-value pairs, making them ideal for structured configuration management.
</p>

**Key Features:**
- Section-based organization with `[Section]` headers
- Key-value pairs with `=` or `:` separators
- Comments with `;` or `#` prefix
- Case-insensitive section and key names
- Support for quoted values
- Boolean and numeric value parsing

**Basic INI Configuration:**
```ini
; Application Configuration File
; Last updated: 2025-09-29

[Application]
Name = MyApplication
Version = 1.0.0
Debug = true
Environment = development
StartupTime = 2025-09-29T10:30:00Z

[Server]
Host = localhost
Port = 8080
Timeout = 30
SSLEnabled = false
MaxConnections = 100
WorkerThreads = 4

[Database]
Host = localhost
Port = 5432
Name = myapp_database
Username = postgres
Password = "secret password with spaces"
PoolSize = 10
ConnectionTimeout = 30
SSLMode = prefer
AutoReconnect = true

[Logging]
Level = info
File = /var/log/application.log
MaxSize = 100MB
Rotate = true
RetentionDays = 30
Format = text
```

**Advanced INI with Multiple Environments:**
```ini
; Multi-environment configuration
; Use sections to separate different deployment environments

[Global]
ApplicationName = Enterprise App
Version = 2.1.0
Company = Acme Corporation
SupportEmail = support@acme.com

[Development]
DebugMode = true
LogLevel = debug
DatabaseHost = localhost
DatabasePort = 5432
DatabaseName = myapp_dev
CacheEnabled = false
SSLRequired = false
ApiRateLimit = 10000

[Staging]
DebugMode = false
LogLevel = info
DatabaseHost = staging-db.acme.com
DatabasePort = 5432
DatabaseName = myapp_staging
CacheEnabled = true
SSLRequired = true
ApiRateLimit = 5000

[Production]
DebugMode = false
LogLevel = warn
DatabaseHost = prod-db.acme.com
DatabasePort = 5432
DatabaseName = myapp_production
CacheEnabled = true
SSLRequired = true
ApiRateLimit = 1000

; Feature flags for different environments
[Features.Development]
Analytics = false
Monitoring = false
Profiling = true
TestMode = true

[Features.Production]
Analytics = true
Monitoring = true
Profiling = false
TestMode = false
```

**Complex INI with Service Configuration:**
```ini
; Enterprise service configuration

[Service]
Name = "Payment Processing Service"
Description = "Handles payment transactions and billing"
Version = 3.2.1
Maintainer = "Platform Team <platform@acme.com>"
StartupType = automatic
RestartPolicy = always

[Network]
BindAddress = 0.0.0.0
Port = 8443
Protocol = https
SSLCertificate = /etc/ssl/certs/service.crt
SSLPrivateKey = /etc/ssl/private/service.key
SSLProtocols = "TLSv1.2,TLSv1.3"
MaxRequestSize = 10MB

[Security]
AuthenticationRequired = true
AuthorizationMode = rbac
TokenExpiration = 3600
PasswordMinLength = 12
PasswordRequireSpecialChars = true
RateLimitingEnabled = true
BruteForceProtection = true

[Database.Primary]
Driver = postgresql
Host = primary-db.acme.com
Port = 5432
Database = payments
Username = payment_service
Password = "${DB_PRIMARY_PASSWORD}"
PoolSize = 25
MaxIdleConnections = 5
ConnectionTimeout = 30
QueryTimeout = 60

[Database.Replica]
Driver = postgresql
Host = replica-db.acme.com
Port = 5432
Database = payments
Username = payment_service_readonly
Password = "${DB_REPLICA_PASSWORD}"
PoolSize = 15
MaxIdleConnections = 3
ReadOnly = true

[Cache]
Engine = redis
Hosts = "cache1.acme.com:6379,cache2.acme.com:6379,cache3.acme.com:6379"
Password = "${REDIS_PASSWORD}"
Database = 0
TTL = 1800
MaxMemory = 512MB
EvictionPolicy = allkeys-lru

[Monitoring]
Enabled = true
MetricsPort = 9090
HealthCheckPath = /health
PrometheusEnabled = true
JaegerEndpoint = http://jaeger.acme.com:14268
LogAggregation = true
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="properties">Properties</h2>
<p>
    <strong>Properties</strong> files are a Java-originated format widely used across platforms for configuration. They use a flat key-value structure with dot notation for hierarchy, making them simple, predictable, and easy to integrate with environment variables and system properties.
</p>

**Key Features:**
- Flat key-value pairs with `=` or `:` separators
- Dot notation for hierarchical organization
- Comments with `#` or `!` prefix
- Unicode escape sequences support
- Multi-line values with backslash continuation
- Automatic trimming of whitespace

**Basic Properties Configuration:**
```properties
# Application Properties
# Generated on 2025-09-29

# Application metadata
app.name=MyApplication
app.version=1.0.0
app.description=A sample application configuration
app.debug=true
app.environment=development
app.build.timestamp=2025-09-29T10:30:00Z

# Server configuration
server.host=localhost
server.port=8080
server.context.path=/api/v1
server.timeout=30000
server.ssl.enabled=false
server.max.connections=100
server.thread.pool.size=10

# Database properties
database.url=jdbc:postgresql://localhost:5432/myapp
database.driver=org.postgresql.Driver
database.username=postgres
database.password=secret123
database.pool.initial.size=5
database.pool.max.size=20
database.pool.min.idle=2
database.connection.timeout=30
database.validation.query=SELECT 1

# Logging configuration
logging.level.root=INFO
logging.level.com.myapp=DEBUG
logging.level.org.springframework=WARN
logging.file.name=/var/log/application.log
logging.file.max.size=100MB
logging.file.max.history=30
logging.pattern.console=%d{yyyy-MM-dd HH:mm:ss} - %msg%n
```

**Advanced Properties with Multi-line Values:**
```properties
# Advanced configuration with multi-line values and complex structures

# Application description (multi-line)
app.description=This is a comprehensive application \n\
                that handles multiple business processes \n\
                including user management, billing, and reporting.

# Database configuration for multiple environments
spring.profiles.active=development

# Development database
spring.datasource.development.url=jdbc:h2:mem:devdb
spring.datasource.development.driver=org.h2.Driver
spring.datasource.development.username=sa
spring.datasource.development.password=

# Production database
spring.datasource.production.url=jdbc:postgresql://prod-db:5432/myapp
spring.datasource.production.driver=org.postgresql.Driver
spring.datasource.production.username=${DB_USER}
spring.datasource.production.password=${DB_PASSWORD}

# JPA/Hibernate configuration
spring.jpa.hibernate.ddl-auto=validate
spring.jpa.show-sql=false
spring.jpa.properties.hibernate.dialect=org.hibernate.dialect.PostgreSQLDialect
spring.jpa.properties.hibernate.format_sql=true
spring.jpa.properties.hibernate.use_sql_comments=true

# Connection pool settings
spring.datasource.hikari.maximum-pool-size=20
spring.datasource.hikari.minimum-idle=5
spring.datasource.hikari.idle-timeout=300000
spring.datasource.hikari.max-lifetime=1200000
spring.datasource.hikari.connection-timeout=20000

# Security configuration
security.jwt.secret=${JWT_SECRET:default-secret-key}
security.jwt.expiration=86400
security.cors.allowed.origins=http://localhost:3000,https://app.example.com
security.cors.allowed.methods=GET,POST,PUT,DELETE,OPTIONS
security.cors.allowed.headers=Content-Type,Authorization,X-Requested-With

# API configuration
api.rate.limit.requests.per.minute=1000
api.rate.limit.burst.size=100
api.documentation.enabled=true
api.documentation.title=My Application API
api.documentation.version=1.0.0
api.documentation.description=RESTful API for MyApplication
```

**Enterprise Properties Configuration:**
```properties
# Enterprise application properties
# Environment: Production
# Last updated: 2025-09-29

# Application identification
app.name=Enterprise Payment System
app.version=3.2.1
app.instance.id=${HOSTNAME:unknown}
app.datacenter=${DATACENTER:us-east-1}
app.environment=production

# Server configuration
server.port=${PORT:8443}
server.servlet.context-path=/api
server.ssl.enabled=true
server.ssl.key-store=/etc/ssl/keystore.p12
server.ssl.key-store-password=${SSL_KEYSTORE_PASSWORD}
server.ssl.key-store-type=PKCS12
server.ssl.protocol=TLS
server.ssl.enabled-protocols=TLSv1.2,TLSv1.3

# Load balancer and clustering
server.use-forward-headers=true
server.forward-headers-strategy=native
management.server.port=8444
management.endpoints.web.exposure.include=health,metrics,info

# Database cluster configuration
spring.datasource.primary.url=${DB_PRIMARY_URL}
spring.datasource.primary.username=${DB_PRIMARY_USER}
spring.datasource.primary.password=${DB_PRIMARY_PASSWORD}
spring.datasource.primary.driver-class-name=org.postgresql.Driver

spring.datasource.secondary.url=${DB_SECONDARY_URL}
spring.datasource.secondary.username=${DB_SECONDARY_USER}
spring.datasource.secondary.password=${DB_SECONDARY_PASSWORD}
spring.datasource.secondary.driver-class-name=org.postgresql.Driver

# Connection pooling
spring.datasource.hikari.maximum-pool-size=50
spring.datasource.hikari.minimum-idle=10
spring.datasource.hikari.connection-timeout=20000
spring.datasource.hikari.idle-timeout=300000
spring.datasource.hikari.max-lifetime=1200000
spring.datasource.hikari.leak-detection-threshold=60000

# Caching configuration
spring.cache.type=redis
spring.redis.host=${REDIS_HOST:localhost}
spring.redis.port=${REDIS_PORT:6379}
spring.redis.password=${REDIS_PASSWORD}
spring.redis.database=0
spring.redis.timeout=2000
spring.redis.lettuce.pool.max-active=20
spring.redis.lettuce.pool.max-idle=8
spring.redis.lettuce.pool.min-idle=2

# Message queue configuration
spring.rabbitmq.host=${RABBITMQ_HOST}
spring.rabbitmq.port=${RABBITMQ_PORT:5672}
spring.rabbitmq.username=${RABBITMQ_USER}
spring.rabbitmq.password=${RABBITMQ_PASSWORD}
spring.rabbitmq.virtual-host=/payments

# Monitoring and observability
management.metrics.export.prometheus.enabled=true
management.tracing.sampling.probability=0.1
logging.level.com.enterprise=INFO
logging.level.org.springframework.security=WARN
logging.appender.file.name=/var/log/enterprise-payment-system.log
logging.appender.file.max-file-size=200MB
logging.appender.file.max-history=50
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="json">JSON</h2>
<p>
    <strong>JSON</strong> (JavaScript Object Notation) is a lightweight, text-based data interchange format that's become the standard for modern web APIs and configuration files. Its hierarchical structure and native support for complex data types make it ideal for sophisticated configuration scenarios.
</p>

**Key Features:**
- Hierarchical object and array structures
- Native support for strings, numbers, booleans, null
- Unicode support
- Comments not officially supported (use description fields)
- Strict syntax validation
- Excellent tooling and editor support

**Basic JSON Configuration:**
```json
{
  "application": {
    "name": "MyApplication",
    "version": "1.0.0",
    "description": "A sample application with JSON configuration",
    "debug": true,
    "environment": "development",
    "build": {
      "timestamp": "2025-09-29T10:30:00Z",
      "commit": "abc123def456",
      "branch": "main"
    }
  },
  "server": {
    "host": "localhost",
    "port": 8080,
    "timeout": 30,
    "ssl": {
      "enabled": false,
      "certificate": null,
      "private_key": null
    },
    "limits": {
      "max_connections": 100,
      "request_timeout": 60,
      "max_request_size": "10MB"
    }
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "myapp_db",
    "username": "postgres",
    "password": "secret123",
    "pool": {
      "initial_size": 5,
      "max_size": 20,
      "min_idle": 2,
      "connection_timeout": 30
    },
    "ssl_mode": "prefer"
  },
  "logging": {
    "level": "info",
    "file": "/var/log/application.log",
    "max_size": "100MB",
    "rotate": true,
    "retention_days": 30,
    "format": "json"
  }
}
```

**Advanced JSON with Arrays and Complex Structures:**
```json
{
  "metadata": {
    "name": "Enterprise Configuration",
    "version": "2.1.0",
    "created": "2025-09-29T10:30:00Z",
    "environment": "production",
    "tags": ["production", "enterprise", "payment-system"]
  },
  "services": {
    "api": {
      "enabled": true,
      "host": "0.0.0.0",
      "port": 8443,
      "ssl": {
        "enabled": true,
        "certificate": "/etc/ssl/certs/api.crt",
        "private_key": "/etc/ssl/private/api.key",
        "protocols": ["TLSv1.2", "TLSv1.3"]
      },
      "cors": {
        "allowed_origins": ["https://app.example.com", "https://admin.example.com"],
        "allowed_methods": ["GET", "POST", "PUT", "DELETE"],
        "allowed_headers": ["Content-Type", "Authorization", "X-API-Key"],
        "expose_headers": ["X-Total-Count", "X-Rate-Limit-Remaining"],
        "credentials": true,
        "max_age": 3600
      },
      "rate_limiting": {
        "enabled": true,
        "requests_per_minute": 1000,
        "burst_limit": 100,
        "skip_successful_requests": false
      }
    },
    "worker": {
      "enabled": true,
      "concurrency": 4,
      "queue_size": 1000,
      "retry_policy": {
        "max_attempts": 3,
        "backoff_multiplier": 2.0,
        "initial_delay": 1000,
        "max_delay": 30000
      }
    }
  },
  "databases": [
    {
      "name": "primary",
      "type": "postgresql",
      "host": "primary-db.example.com",
      "port": 5432,
      "database": "production",
      "credentials": {
        "username": "app_user",
        "password": "${DB_PRIMARY_PASSWORD}"
      },
      "pool": {
        "initial_size": 10,
        "max_size": 50,
        "min_idle": 5,
        "connection_timeout": 30,
        "idle_timeout": 300,
        "max_lifetime": 1200
      },
      "ssl": {
        "mode": "require",
        "ca_cert": "/etc/ssl/certs/db-ca.crt"
      },
      "read_only": false
    },
    {
      "name": "replica",
      "type": "postgresql",
      "host": "replica-db.example.com",
      "port": 5432,
      "database": "production",
      "credentials": {
        "username": "app_readonly",
        "password": "${DB_REPLICA_PASSWORD}"
      },
      "pool": {
        "initial_size": 5,
        "max_size": 25,
        "min_idle": 2,
        "connection_timeout": 30,
        "idle_timeout": 300,
        "max_lifetime": 1200
      },
      "ssl": {
        "mode": "require",
        "ca_cert": "/etc/ssl/certs/db-ca.crt"
      },
      "read_only": true
    }
  ],
  "cache": {
    "provider": "redis",
    "cluster": {
      "nodes": [
        {"host": "cache1.example.com", "port": 6379},
        {"host": "cache2.example.com", "port": 6379},
        {"host": "cache3.example.com", "port": 6379}
      ],
      "password": "${REDIS_PASSWORD}",
      "database": 0
    },
    "settings": {
      "default_ttl": 3600,
      "max_memory": "512MB",
      "eviction_policy": "allkeys-lru",
      "persistence": {
        "enabled": true,
        "strategy": "rdb",
        "interval": 900
      }
    }
  },
  "monitoring": {
    "metrics": {
      "enabled": true,
      "port": 9090,
      "path": "/metrics",
      "collectors": ["cpu", "memory", "disk", "network", "application"]
    },
    "tracing": {
      "enabled": true,
      "sampling_rate": 0.1,
      "jaeger": {
        "endpoint": "http://jaeger.example.com:14268",
        "service_name": "payment-api"
      }
    },
    "health_checks": {
      "enabled": true,
      "endpoint": "/health",
      "checks": [
        {"name": "database", "timeout": 5000},
        {"name": "cache", "timeout": 2000},
        {"name": "external_api", "timeout": 10000}
      ]
    }
  },
  "features": {
    "payment_processing": {
      "enabled": true,
      "providers": ["stripe", "paypal", "square"],
      "default_provider": "stripe",
      "retry_failed_payments": true,
      "max_retry_attempts": 3
    },
    "user_analytics": {
      "enabled": true,
      "anonymize_ip": true,
      "retention_days": 90,
      "export_enabled": false
    },
    "advanced_reporting": {
      "enabled": false,
      "scheduled_reports": [],
      "custom_dashboards": false
    }
  },
  "integrations": {
    "email": {
      "provider": "sendgrid",
      "api_key": "${SENDGRID_API_KEY}",
      "from_address": "noreply@example.com",
      "templates": {
        "welcome": "d-abc123",
        "password_reset": "d-def456",
        "payment_confirmation": "d-ghi789"
      }
    },
    "sms": {
      "provider": "twilio",
      "account_sid": "${TWILIO_ACCOUNT_SID}",
      "auth_token": "${TWILIO_AUTH_TOKEN}",
      "from_number": "+1234567890"
    },
    "external_apis": [
      {
        "name": "payment_gateway",
        "base_url": "https://api.payment-provider.com",
        "api_key": "${PAYMENT_API_KEY}",
        "timeout": 30000,
        "retry_attempts": 3
      },
      {
        "name": "fraud_detection",
        "base_url": "https://api.fraud-detector.com",
        "api_key": "${FRAUD_API_KEY}",
        "timeout": 5000,
        "retry_attempts": 1
      }
    ]
  }
}
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="noml">NOML</h2>
<p>
    <strong>NOML</strong> (Nested Object Markup Language) is a modern configuration language that extends TOML with dynamic features. It combines TOML's human-readable syntax with powerful capabilities like environment variable interpolation, file includes, and native type extensions, making it ideal for complex, maintainable configurations.
</p>

**Key Features:**
- TOML-compatible base syntax
- Environment variable interpolation with `${VAR}`
- File inclusion with `include "path"`
- Native type extensions: `@duration()`, `@size()`, `@url()`
- Dynamic resolution and validation
- Nested object support
- Comments and documentation

**Basic NOML Configuration:**
```noml
# Application Configuration
# NOML format with dynamic features

[app]
name = "MyApplication"
version = "1.0.0"
debug = ${DEBUG:false}
environment = ${ENVIRONMENT:"development"}
build_time = @datetime("2025-09-29T10:30:00Z")

[server]
host = ${SERVER_HOST:"localhost"}
port = ${PORT:8080}
timeout = @duration("30s")
max_request_size = @size("10MB")
ssl_enabled = ${SSL_ENABLED:false}

[database]
host = ${DB_HOST:"localhost"}
port = ${DB_PORT:5432}
name = ${DB_NAME:"myapp"}
username = ${DB_USER:"postgres"}
password = ${DB_PASSWORD}
pool_size = ${DB_POOL_SIZE:10}
connection_timeout = @duration("30s")
ssl_mode = ${DB_SSL_MODE:"prefer"}

[logging]
level = ${LOG_LEVEL:"info"}
file = ${LOG_FILE:"/var/log/app.log"}
max_size = @size("100MB")
rotate = ${LOG_ROTATE:true}
retention = @duration("30d")

# Include external configuration
include "logging.noml"
include "features.noml"
```

**Advanced NOML with Dynamic Features:**
```noml
# Enterprise NOML Configuration
# Demonstrates advanced features and dynamic resolution

[metadata]
name = "Enterprise Payment System"
version = "3.2.1"
created = @datetime("now")
environment = ${DEPLOYMENT_ENV:"production"}
datacenter = ${DATACENTER:"us-east-1"}
instance_id = ${HOSTNAME:"unknown"}

# Server configuration with environment-specific values
[server]
bind_address = ${BIND_ADDRESS:"0.0.0.0"}
port = ${SERVER_PORT:8443}
context_path = "/api/v1"
request_timeout = @duration("${REQUEST_TIMEOUT:60s}")
max_connections = ${MAX_CONNECTIONS:1000}
worker_threads = ${WORKER_THREADS:4}

[server.ssl]
enabled = ${SSL_ENABLED:true}
certificate = ${SSL_CERT_PATH:"/etc/ssl/certs/server.crt"}
private_key = ${SSL_KEY_PATH:"/etc/ssl/private/server.key"}
protocols = ["TLSv1.2", "TLSv1.3"]
cipher_suites = include "ssl-ciphers.noml"

# Database cluster configuration
[database.primary]
url = @url("${DB_PRIMARY_URL}")
driver = "postgresql"
max_pool_size = ${DB_PRIMARY_POOL:25}
min_idle = ${DB_PRIMARY_MIN_IDLE:5}
connection_timeout = @duration("30s")
query_timeout = @duration("60s")
idle_timeout = @duration("5m")
max_lifetime = @duration("20m")

[database.replica]
url = @url("${DB_REPLICA_URL}")
driver = "postgresql"
max_pool_size = ${DB_REPLICA_POOL:15}
min_idle = ${DB_REPLICA_MIN_IDLE:3}
read_only = true
connection_timeout = @duration("30s")

# Cache configuration with dynamic sizing
[cache]
engine = ${CACHE_ENGINE:"redis"}
cluster_nodes = [
  @url("redis://${REDIS_HOST1:cache1.example.com}:${REDIS_PORT:6379}"),
  @url("redis://${REDIS_HOST2:cache2.example.com}:${REDIS_PORT:6379}"),
  @url("redis://${REDIS_HOST3:cache3.example.com}:${REDIS_PORT:6379}")
]
password = ${REDIS_PASSWORD}
database = ${REDIS_DB:0}
default_ttl = @duration("1h")
max_memory = @size("${REDIS_MAX_MEMORY:512MB}")
eviction_policy = "allkeys-lru"

# API configuration with rate limiting
[api]
version = "v1"
base_path = "/api/v1"
rate_limit = {
  requests_per_minute = ${API_RATE_LIMIT:1000},
  burst_limit = ${API_BURST_LIMIT:100},
  window = @duration("1m")
}
cors = {
  allowed_origins = [${CORS_ORIGINS:"*"}],
  allowed_methods = ["GET", "POST", "PUT", "DELETE"],
  allowed_headers = ["Content-Type", "Authorization"],
  credentials = ${CORS_CREDENTIALS:true},
  max_age = @duration("1h")
}

# Monitoring and observability
[monitoring]
metrics_enabled = ${METRICS_ENABLED:true}
metrics_port = ${METRICS_PORT:9090}
tracing_enabled = ${TRACING_ENABLED:true}
tracing_sample_rate = ${TRACING_SAMPLE_RATE:0.1}

[monitoring.jaeger]
endpoint = @url("${JAEGER_ENDPOINT:http://jaeger:14268}")
service_name = "payment-api"
tags = {
  version = "3.2.1",
  environment = ${DEPLOYMENT_ENV:"production"},
  datacenter = ${DATACENTER:"us-east-1"}
}

# Feature flags with environment overrides
[features]
payment_processing = ${FEATURE_PAYMENTS:true}
user_analytics = ${FEATURE_ANALYTICS:true}
advanced_reporting = ${FEATURE_REPORTING:false}
debug_mode = ${DEBUG_MODE:false}
maintenance_mode = ${MAINTENANCE_MODE:false}

# External service integrations
[integrations.email]
provider = "sendgrid"
api_key = ${SENDGRID_API_KEY}
from_address = ${EMAIL_FROM:"noreply@example.com"}
timeout = @duration("30s")
retry_attempts = ${EMAIL_RETRY_ATTEMPTS:3}

[integrations.payment_gateway]
base_url = @url("${PAYMENT_GATEWAY_URL}")
api_key = ${PAYMENT_GATEWAY_KEY}
timeout = @duration("${PAYMENT_TIMEOUT:30s}")
retry_attempts = ${PAYMENT_RETRY_ATTEMPTS:3}
webhook_secret = ${PAYMENT_WEBHOOK_SECRET}

# Logging configuration
[logging]
level = ${LOG_LEVEL:"info"}
format = ${LOG_FORMAT:"json"}
output = ${LOG_OUTPUT:"file"}
file = ${LOG_FILE:"/var/log/payment-api.log"}
max_size = @size("${LOG_MAX_SIZE:200MB}")
max_files = ${LOG_MAX_FILES:10}
compress = ${LOG_COMPRESS:true}

# Include environment-specific overrides
include "environments/${DEPLOYMENT_ENV:production}.noml"
include "secrets.noml"
```

**NOML with Includes and Modular Configuration:**
```noml
# Main configuration file (app.noml)
# Demonstrates modular configuration with includes

[app]
name = "Modular Application"
version = "2.0.0"
environment = ${ENVIRONMENT:"development"}

# Include base configurations
include "configs/database.noml"
include "configs/server.noml"
include "configs/logging.noml"

# Include environment-specific configuration
include "environments/${ENVIRONMENT}.noml"

# Include optional local overrides
include "local.noml"  # This file may not exist

# Feature toggles
[features]
api_v2 = ${ENABLE_API_V2:false}
metrics = ${ENABLE_METRICS:true}
tracing = ${ENABLE_TRACING:false}

# Security configuration
[security]
jwt_secret = ${JWT_SECRET}
jwt_expiry = @duration("${JWT_EXPIRY:24h}")
password_min_length = ${PASSWORD_MIN_LENGTH:12}
rate_limiting = {
  enabled = ${RATE_LIMITING:true},
  window = @duration("1m"),
  max_requests = ${RATE_LIMIT_REQUESTS:100}
}

# File: configs/database.noml
[database]
driver = "postgresql"
host = ${DB_HOST:"localhost"}
port = ${DB_PORT:5432}
name = ${DB_NAME:"myapp"}
username = ${DB_USER:"postgres"}
password = ${DB_PASSWORD}
ssl_mode = ${DB_SSL_MODE:"prefer"}

[database.pool]
max_size = ${DB_POOL_MAX:20}
min_idle = ${DB_POOL_MIN:5}
connection_timeout = @duration("30s")
idle_timeout = @duration("10m")
max_lifetime = @duration("30m")

# File: environments/production.noml
[app]
debug = false

[server]
host = "0.0.0.0"
port = 8443
ssl_enabled = true

[database]
ssl_mode = "require"
pool.max_size = 50

[logging]
level = "warn"
format = "json"

[features]
metrics = true
tracing = true
api_v2 = true
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="toml">TOML</h2>
<p>
    <strong>TOML</strong> (Tom's Obvious, Minimal Language) is a configuration file format that aims to be easy to read and write due to obvious semantics. It's designed to map unambiguously to a hash table and is particularly popular in the Rust ecosystem and for modern application configuration.
</p>

**Key Features:**
- Clear, readable syntax
- Strong typing (strings, integers, floats, booleans, dates)
- Nested tables and arrays
- Comments with `#` prefix
- Date and time support
- Multi-line strings
- Array of tables for complex structures

**Basic TOML Configuration:**
```toml
# Application Configuration
# TOML format example

# Global settings
app_name = "MyApplication"
version = "1.0.0"
debug = true
environment = "development"
created = 2025-09-29T10:30:00Z

# Server configuration
[server]
host = "localhost"
port = 8080
timeout = 30
ssl_enabled = false
max_connections = 100
worker_threads = 4

# Database settings
[database]
host = "localhost"
port = 5432
name = "myapp_db"
username = "postgres"
password = "secret123"
pool_size = 10
connection_timeout = 30
ssl_mode = "prefer"
auto_reconnect = true

# Logging configuration
[logging]
level = "info"
file = "/var/log/application.log"
max_size = "100MB"
rotate = true
retention_days = 30
format = "text"

# Array of allowed hosts
allowed_hosts = ["localhost", "127.0.0.1", "::1"]

# Feature flags
[features]
analytics = false
monitoring = true
api_versioning = true
debug_endpoints = false
```

**Advanced TOML with Complex Structures:**
```toml
# Enterprise TOML Configuration
# Demonstrates advanced TOML features

title = "Enterprise Application Configuration"
version = "3.2.1"
description = """
Enterprise payment processing system
with advanced monitoring and analytics.
Supports multiple payment providers.
"""

[metadata]
owner = "Platform Team"
email = "platform@example.com"
created = 2025-09-29T10:30:00Z
last_updated = 2025-09-29T15:45:00Z
tags = ["production", "payment", "enterprise"]

# Server configuration with nested tables
[server]
host = "0.0.0.0"
port = 8443
context_path = "/api/v1"
request_timeout = 60
max_connections = 1000

# SSL configuration as nested table
[server.ssl]
enabled = true
certificate = "/etc/ssl/certs/server.crt"
private_key = "/etc/ssl/private/server.key"
protocols = ["TLSv1.2", "TLSv1.3"]
cipher_suites = [
  "TLS_AES_256_GCM_SHA384",
  "TLS_CHACHA20_POLY1305_SHA256",
  "TLS_AES_128_GCM_SHA256"
]

# CORS configuration
[server.cors]
allowed_origins = ["https://app.example.com", "https://admin.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Content-Type", "Authorization", "X-API-Key"]
credentials = true
max_age = 3600

# Database configuration with multiple environments
[database.primary]
driver = "postgresql"
host = "primary-db.example.com"
port = 5432
database = "production"
username = "app_user"
password = "${DB_PRIMARY_PASSWORD}"
max_pool_size = 25
min_idle_connections = 5
connection_timeout = 30
query_timeout = 60
idle_timeout = 300
max_lifetime = 1200
ssl_mode = "require"
ssl_ca_cert = "/etc/ssl/certs/db-ca.crt"

[database.replica]
driver = "postgresql"
host = "replica-db.example.com"
port = 5432
database = "production"
username = "app_readonly"
password = "${DB_REPLICA_PASSWORD}"
max_pool_size = 15
min_idle_connections = 3
read_only = true
ssl_mode = "require"

# Cache configuration
[cache]
engine = "redis"
password = "${REDIS_PASSWORD}"
database = 0
default_ttl = 3600
max_memory = "512MB"
eviction_policy = "allkeys-lru"

# Redis cluster nodes
[[cache.nodes]]
host = "cache1.example.com"
port = 6379
role = "master"

[[cache.nodes]]
host = "cache2.example.com"
port = 6379
role = "slave"

[[cache.nodes]]
host = "cache3.example.com"
port = 6379
role = "slave"

# API configuration
[api]
version = "v1"
base_path = "/api/v1"
documentation_enabled = true

[api.rate_limiting]
enabled = true
requests_per_minute = 1000
burst_limit = 100
skip_successful_requests = false

# Array of tables for external integrations
[[integrations]]
name = "payment_gateway"
type = "http"
base_url = "https://api.payment-provider.com"
api_key = "${PAYMENT_API_KEY}"
timeout = 30
retry_attempts = 3
health_check_path = "/health"

[[integrations]]
name = "fraud_detection"
type = "http"
base_url = "https://api.fraud-detector.com"
api_key = "${FRAUD_API_KEY}"
timeout = 5
retry_attempts = 1
health_check_path = "/status"

[[integrations]]
name = "email_service"
type = "smtp"
host = "smtp.sendgrid.net"
port = 587
username = "apikey"
password = "${SENDGRID_API_KEY}"
from_address = "noreply@example.com"
tls_enabled = true

# Monitoring configuration
[monitoring]
metrics_enabled = true
metrics_port = 9090
metrics_path = "/metrics"
health_check_enabled = true
health_check_path = "/health"

[monitoring.prometheus]
enabled = true
collectors = ["cpu", "memory", "disk", "network", "application"]
scrape_interval = "15s"

[monitoring.jaeger]
enabled = true
endpoint = "http://jaeger.example.com:14268"
service_name = "payment-api"
sampling_rate = 0.1

# Logging with multiple outputs
[logging]
level = "info"
format = "json"

[[logging.outputs]]
type = "file"
path = "/var/log/application.log"
max_size = "200MB"
max_files = 10
compress = true

[[logging.outputs]]
type = "stdout"
format = "text"
colors = true

[[logging.outputs]]
type = "elasticsearch"
host = "elasticsearch.example.com"
port = 9200
index = "application-logs"
auth_enabled = true
username = "logger"
password = "${ELASTICSEARCH_PASSWORD}"

# Security configuration
[security]
jwt_secret = "${JWT_SECRET}"
jwt_expiration = 86400
password_min_length = 12
password_require_special_chars = true
brute_force_protection = true
max_login_attempts = 5
lockout_duration = 900

# Feature flags
[features]
payment_processing = true
user_analytics = true
advanced_reporting = false
debug_mode = false
maintenance_mode = false
api_v2_enabled = false
```

**TOML with Time and Date Examples:**
```toml
# TOML Date and Time Examples
# Demonstrates TOML's native date/time support

[application]
name = "DateTime Demo"
version = "1.0.0"

# Various date and time formats
released = 2025-09-29  # Date only
launched = 10:30:00    # Time only
created = 2025-09-29T10:30:00Z  # RFC 3339 UTC
updated = 2025-09-29T10:30:00+02:00  # RFC 3339 with timezone
last_backup = 2025-09-29T10:30:00.123Z  # With milliseconds

[schedule]
# Scheduled tasks with specific times
daily_backup = 02:00:00
weekly_report = 2025-10-06T09:00:00Z
monthly_maintenance = 2025-11-01T01:00:00Z

# Maintenance windows
[[maintenance_windows]]
start = 2025-10-15T02:00:00Z
end = 2025-10-15T04:00:00Z
description = "Database maintenance"

[[maintenance_windows]]
start = 2025-11-01T01:00:00Z
end = 2025-11-01T03:00:00Z
description = "System updates"

[timeouts]
connection = 30  # seconds
read = 60       # seconds
write = 45      # seconds
idle = 300      # seconds

# Durations in different units
[durations]
cache_ttl = 3600        # seconds
session_timeout = 1800  # seconds
token_lifetime = 86400  # seconds
backup_retention = 2592000  # seconds (30 days)
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="xml">XML</h2>
<p>
    <strong>XML</strong> (eXtensible Markup Language) is a markup language that provides a flexible format for structured data. While more verbose than other formats, XML excels in complex hierarchical configurations, enterprise environments, and scenarios requiring schema validation and namespaces.
</p>

**Key Features:**
- Hierarchical structure with nested elements
- Attributes for metadata
- Namespace support for modular configurations
- Schema validation (XSD)
- Comments and processing instructions
- Mixed content support
- Self-documenting structure

**Basic XML Configuration:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- Application Configuration -->
<!-- Last updated: 2025-09-29 -->
<configuration>
  <application>
    <name>MyApplication</name>
    <version>1.0.0</version>
    <description>A sample application with XML configuration</description>
    <debug>true</debug>
    <environment>development</environment>
    <build>
      <timestamp>2025-09-29T10:30:00Z</timestamp>
      <commit>abc123def456</commit>
      <branch>main</branch>
    </build>
  </application>

  <server>
    <host>localhost</host>
    <port>8080</port>
    <timeout>30</timeout>
    <ssl enabled="false">
      <certificate></certificate>
      <private_key></private_key>
    </ssl>
    <limits>
      <max_connections>100</max_connections>
      <request_timeout>60</request_timeout>
      <max_request_size>10MB</max_request_size>
    </limits>
  </server>

  <database>
    <host>localhost</host>
    <port>5432</port>
    <name>myapp_db</name>
    <username>postgres</username>
    <password>secret123</password>
    <pool>
      <initial_size>5</initial_size>
      <max_size>20</max_size>
      <min_idle>2</min_idle>
      <connection_timeout>30</connection_timeout>
    </pool>
    <ssl_mode>prefer</ssl_mode>
  </database>

  <logging>
    <level>info</level>
    <file>/var/log/application.log</file>
    <max_size>100MB</max_size>
    <rotate>true</rotate>
    <retention_days>30</retention_days>
    <format>json</format>
  </logging>
</configuration>
```

**Advanced XML with Attributes and Namespaces:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- Enterprise XML Configuration -->
<!-- Demonstrates advanced XML features -->
<config:configuration 
  xmlns:config="http://example.com/config/v1"
  xmlns:security="http://example.com/security/v1"
  xmlns:monitoring="http://example.com/monitoring/v1"
  version="3.2.1"
  environment="production"
  created="2025-09-29T10:30:00Z">

  <config:metadata>
    <config:name>Enterprise Payment System</config:name>
    <config:version>3.2.1</config:version>
    <config:description>
      Enterprise payment processing system with
      advanced monitoring and analytics capabilities.
    </config:description>
    <config:owner email="platform@example.com">Platform Team</config:owner>
    <config:tags>
      <config:tag>production</config:tag>
      <config:tag>payment</config:tag>
      <config:tag>enterprise</config:tag>
    </config:tags>
  </config:metadata>

  <config:services>
    <config:service name="api" enabled="true">
      <config:network>
        <config:host>0.0.0.0</config:host>
        <config:port>8443</config:port>
        <config:context_path>/api/v1</config:path>
      </config:network>
      
      <security:ssl enabled="true">
        <security:certificate>/etc/ssl/certs/api.crt</security:certificate>
        <security:private_key>/etc/ssl/private/api.key</security:private_key>
        <security:protocols>
          <security:protocol>TLSv1.2</security:protocol>
          <security:protocol>TLSv1.3</security:protocol>
        </security:protocols>
        <security:cipher_suites>
          <security:cipher>TLS_AES_256_GCM_SHA384</security:cipher>
          <security:cipher>TLS_CHACHA20_POLY1305_SHA256</security:cipher>
        </security:cipher_suites>
      </security:ssl>

      <config:cors>
        <config:allowed_origins>
          <config:origin>https://app.example.com</config:origin>
          <config:origin>https://admin.example.com</config:origin>
        </config:allowed_origins>
        <config:allowed_methods>
          <config:method>GET</config:method>
          <config:method>POST</config:method>
          <config:method>PUT</config:method>
          <config:method>DELETE</config:method>
        </config:allowed_methods>
        <config:allowed_headers>
          <config:header>Content-Type</config:header>
          <config:header>Authorization</config:header>
          <config:header>X-API-Key</config:header>
        </config:allowed_headers>
        <config:credentials>true</config:credentials>
        <config:max_age>3600</config:max_age>
      </config:cors>

      <config:rate_limiting enabled="true">
        <config:requests_per_minute>1000</config:requests_per_minute>
        <config:burst_limit>100</config:burst_limit>
        <config:skip_successful_requests>false</config:skip_successful_requests>
      </config:rate_limiting>
    </config:service>

    <config:service name="worker" enabled="true">
      <config:concurrency>4</config:concurrency>
      <config:queue_size>1000</config:queue_size>
      <config:retry_policy>
        <config:max_attempts>3</config:max_attempts>
        <config:backoff_multiplier>2.0</config:backoff_multiplier>
        <config:initial_delay>1000</config:initial_delay>
        <config:max_delay>30000</config:max_delay>
      </config:retry_policy>
    </config:service>
  </config:services>

  <config:databases>
    <config:database name="primary" type="postgresql" read_only="false">
      <config:connection>
        <config:host>primary-db.example.com</config:host>
        <config:port>5432</config:port>
        <config:database>production</config:database>
        <config:username>app_user</config:username>
        <config:password>${DB_PRIMARY_PASSWORD}</config:password>
      </config:connection>
      <config:pool>
        <config:initial_size>10</config:initial_size>
        <config:max_size>50</config:max_size>
        <config:min_idle>5</config:min_idle>
        <config:connection_timeout>30</config:connection_timeout>
        <config:idle_timeout>300</config:idle_timeout>
        <config:max_lifetime>1200</config:max_lifetime>
      </config:pool>
      <security:ssl>
        <security:mode>require</security:mode>
        <security:ca_cert>/etc/ssl/certs/db-ca.crt</security:ca_cert>
      </security:ssl>
    </config:database>

    <config:database name="replica" type="postgresql" read_only="true">
      <config:connection>
        <config:host>replica-db.example.com</config:host>
        <config:port>5432</config:port>
        <config:database>production</config:database>
        <config:username>app_readonly</config:username>
        <config:password>${DB_REPLICA_PASSWORD}</config:password>
      </config:connection>
      <config:pool>
        <config:initial_size>5</config:initial_size>
        <config:max_size>25</config:max_size>
        <config:min_idle>2</config:min_idle>
        <config:connection_timeout>30</config:connection_timeout>
        <config:idle_timeout>300</config:idle_timeout>
        <config:max_lifetime>1200</config:max_lifetime>
      </config:pool>
      <security:ssl>
        <security:mode>require</security:mode>
        <security:ca_cert>/etc/ssl/certs/db-ca.crt</security:ca_cert>
      </security:ssl>
    </config:database>
  </config:databases>

  <config:cache provider="redis">
    <config:cluster>
      <config:node host="cache1.example.com" port="6379" role="master"/>
      <config:node host="cache2.example.com" port="6379" role="slave"/>
      <config:node host="cache3.example.com" port="6379" role="slave"/>
      <config:password>${REDIS_PASSWORD}</config:password>
      <config:database>0</config:database>
    </config:cluster>
    <config:settings>
      <config:default_ttl>3600</config:default_ttl>
      <config:max_memory>512MB</config:max_memory>
      <config:eviction_policy>allkeys-lru</config:eviction_policy>
      <config:persistence enabled="true">
        <config:strategy>rdb</config:strategy>
        <config:interval>900</config:interval>
      </config:persistence>
    </config:settings>
  </config:cache>

  <monitoring:monitoring>
    <monitoring:metrics enabled="true">
      <monitoring:port>9090</monitoring:port>
      <monitoring:path>/metrics</monitoring:path>
      <monitoring:collectors>
        <monitoring:collector>cpu</monitoring:collector>
        <monitoring:collector>memory</monitoring:collector>
        <monitoring:collector>disk</monitoring:collector>
        <monitoring:collector>network</monitoring:collector>
        <monitoring:collector>application</monitoring:collector>
      </monitoring:collectors>
    </monitoring:metrics>
    
    <monitoring:tracing enabled="true">
      <monitoring:sampling_rate>0.1</monitoring:sampling_rate>
      <monitoring:jaeger>
        <monitoring:endpoint>http://jaeger.example.com:14268</monitoring:endpoint>
        <monitoring:service_name>payment-api</monitoring:service_name>
      </monitoring:jaeger>
    </monitoring:tracing>
    
    <monitoring:health_checks enabled="true">
      <monitoring:endpoint>/health</monitoring:endpoint>
      <monitoring:checks>
        <monitoring:check name="database" timeout="5000"/>
        <monitoring:check name="cache" timeout="2000"/>
        <monitoring:check name="external_api" timeout="10000"/>
      </monitoring:checks>
    </monitoring:health_checks>
  </monitoring:monitoring>

  <config:integrations>
    <config:integration name="email" type="smtp">
      <config:provider>sendgrid</config:provider>
      <config:api_key>${SENDGRID_API_KEY}</config:api_key>
      <config:from_address>noreply@example.com</config:from_address>
      <config:templates>
        <config:template name="welcome" id="d-abc123"/>
        <config:template name="password_reset" id="d-def456"/>
        <config:template name="payment_confirmation" id="d-ghi789"/>
      </config:templates>
    </config:integration>

    <config:integration name="payment_gateway" type="http">
      <config:base_url>https://api.payment-provider.com</config:base_url>
      <config:api_key>${PAYMENT_API_KEY}</config:api_key>
      <config:timeout>30000</config:timeout>
      <config:retry_attempts>3</config:retry_attempts>
      <config:webhook_secret>${PAYMENT_WEBHOOK_SECRET}</config:webhook_secret>
    </config:integration>
  </config:integrations>

  <config:features>
    <config:feature name="payment_processing" enabled="true">
      <config:providers>
        <config:provider name="stripe" default="true"/>
        <config:provider name="paypal" default="false"/>
        <config:provider name="square" default="false"/>
      </config:providers>
      <config:retry_failed_payments>true</config:retry_failed_payments>
      <config:max_retry_attempts>3</config:max_retry_attempts>
    </config:feature>
    
    <config:feature name="user_analytics" enabled="true">
      <config:anonymize_ip>true</config:anonymize_ip>
      <config:retention_days>90</config:retention_days>
      <config:export_enabled>false</config:export_enabled>
    </config:feature>
    
    <config:feature name="advanced_reporting" enabled="false">
      <config:scheduled_reports/>
      <config:custom_dashboards>false</config:custom_dashboards>
    </config:feature>
  </config:features>
</config:configuration>
```

**XML with CDATA and Processing Instructions:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<?xml-stylesheet type="text/xsl" href="config-transform.xsl"?>
<!-- XML Configuration with CDATA and Processing Instructions -->
<configuration>
  <application name="Advanced XML Demo" version="1.0.0">
    <description>
      <![CDATA[
        This is a complex application configuration that demonstrates
        advanced XML features including CDATA sections, processing
        instructions, and mixed content.
        
        Features:
        - Multi-line descriptions
        - SQL queries in configuration
        - Complex formatting
        - Special characters: <>&"'
      ]]>
    </description>
  </application>

  <database>
    <queries>
      <query name="user_stats">
        <![CDATA[
          SELECT 
            u.id,
            u.username,
            COUNT(o.id) as order_count,
            SUM(o.total) as total_spent
          FROM users u
          LEFT JOIN orders o ON u.id = o.user_id
          WHERE u.active = true
            AND o.created_at >= ?
          GROUP BY u.id, u.username
          ORDER BY total_spent DESC
          LIMIT 100
        ]]>
      </query>
      
      <query name="daily_report">
        <![CDATA[
          WITH daily_stats AS (
            SELECT 
              DATE(created_at) as date,
              COUNT(*) as transactions,
              SUM(amount) as volume
            FROM payments 
            WHERE status = 'completed'
              AND created_at >= CURRENT_DATE - INTERVAL '30 days'
            GROUP BY DATE(created_at)
          )
          SELECT * FROM daily_stats ORDER BY date DESC;
        ]]>
      </query>
    </queries>
  </database>

  <logging>
    <pattern>
      <![CDATA[
        %d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n
      ]]>
    </pattern>
    
    <json_pattern>
      <![CDATA[
        {
          "timestamp": "%d{yyyy-MM-dd'T'HH:mm:ss.SSSZ}",
          "level": "%level",
          "thread": "%thread",
          "logger": "%logger",
          "message": "%msg",
          "mdc": "%X"
        }
      ]]>
    </json_pattern>
  </logging>

  <security>
    <cors_policy>
      <![CDATA[
        allow-origin: https://app.example.com https://admin.example.com;
        allow-methods: GET, POST, PUT, DELETE, OPTIONS;
        allow-headers: Content-Type, Authorization, X-API-Key;
        allow-credentials: true;
        max-age: 3600;
      ]]>
    </cors_policy>
  </security>
</configuration>
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>

<h2 id="hcl">HCL</h2>
<p>
    <strong>HCL</strong> (HashiCorp Configuration Language) is a structured configuration language created by HashiCorp. It's designed to be both human and machine-friendly, with a syntax that's more concise than JSON but more structured than YAML. HCL is widely used in infrastructure-as-code tools like Terraform.
</p>

**Key Features:**
- Block-based structure with labels
- Variable interpolation with `${...}`
- Functions and expressions
- Comments with `#`, `//`, or `/* */`
- Heredocs for multi-line strings
- Lists and maps as first-class types
- Conditional expressions
- For expressions and iteration

**Basic HCL Configuration:**
```hcl
# Application Configuration
# HCL format example

# Global variables
variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "development"
}

variable "debug_enabled" {
  description = "Enable debug mode"
  type        = bool
  default     = true
}

# Application configuration block
application {
  name        = "MyApplication"
  version     = "1.0.0"
  description = "A sample application with HCL configuration"
  debug       = var.debug_enabled
  environment = var.environment
  
  build {
    timestamp = "2025-09-29T10:30:00Z"
    commit    = "abc123def456"
    branch    = "main"
  }
}

# Server configuration
server {
  host    = "localhost"
  port    = 8080
  timeout = 30
  
  ssl {
    enabled     = false
    certificate = null
    private_key = null
  }
  
  limits {
    max_connections    = 100
    request_timeout    = 60
    max_request_size   = "10MB"
  }
}

# Database configuration
database {
  host     = "localhost"
  port     = 5432
  name     = "myapp_db"
  username = "postgres"
  password = "secret123"
  
  pool {
    initial_size       = 5
    max_size          = 20
    min_idle          = 2
    connection_timeout = 30
  }
  
  ssl_mode = "prefer"
}

# Logging configuration
logging {
  level           = "info"
  file            = "/var/log/application.log"
  max_size        = "100MB"
  rotate          = true
  retention_days  = 30
  format          = "json"
}

# Feature flags
features = {
  analytics      = false
  monitoring     = true
  api_versioning = true
  debug_endpoints = var.debug_enabled
}

# List of allowed hosts
allowed_hosts = ["localhost", "127.0.0.1", "::1"]
```

**Advanced HCL with Variables and Functions:**
```hcl
# Advanced HCL Configuration
# Demonstrates variables, functions, and complex structures

# Local values for computed configurations
locals {
  environment = var.environment
  is_production = local.environment == "production"
  
  # Computed database URL
  database_url = "postgresql://${var.db_username}:${var.db_password}@${var.db_host}:${var.db_port}/${var.db_name}"
  
  # Environment-specific settings
  server_config = local.is_production ? {
    workers = 10
    debug   = false
    ssl     = true
  } : {
    workers = 2
    debug   = true
    ssl     = false
  }
  
  # Common tags
  common_tags = {
    Environment = local.environment
    Application = "enterprise-payment-system"
    Team        = "platform"
    CreatedBy   = "config-management"
  }
}

# Variables with validation
variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "development"
  
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "Environment must be development, staging, or production."
  }
}

variable "db_host" {
  description = "Database host"
  type        = string
  default     = "localhost"
}

variable "db_port" {
  description = "Database port"
  type        = number
  default     = 5432
  
  validation {
    condition     = var.db_port > 0 && var.db_port <= 65535
    error_message = "Database port must be between 1 and 65535."
  }
}

variable "db_username" {
  description = "Database username"
  type        = string
  sensitive   = true
}

variable "db_password" {
  description = "Database password"
  type        = string
  sensitive   = true
}

variable "db_name" {
  description = "Database name"
  type        = string
  default     = "myapp"
}

# Application metadata
metadata {
  name        = "Enterprise Payment System"
  version     = "3.2.1"
  description = <<-EOT
    Enterprise payment processing system with
    advanced monitoring and analytics capabilities.
    
    Features:
    - Multi-provider payment processing
    - Real-time fraud detection
    - Comprehensive audit logging
    - High availability deployment
  EOT
  
  owner = {
    team  = "Platform Team"
    email = "platform@example.com"
  }
  
  tags = merge(local.common_tags, {
    Version = "3.2.1"
    Type    = "payment-processor"
  })
}

# Server configuration with conditionals
server {
  host         = local.is_production ? "0.0.0.0" : "localhost"
  port         = local.is_production ? 8443 : 8080
  workers      = local.server_config.workers
  context_path = "/api/v1"
  
  # SSL configuration
  ssl {
    enabled     = local.server_config.ssl
    certificate = local.is_production ? "/etc/ssl/certs/server.crt" : null
    private_key = local.is_production ? "/etc/ssl/private/server.key" : null
    protocols   = ["TLSv1.2", "TLSv1.3"]
  }
  
  # CORS configuration
  cors {
    allowed_origins = local.is_production ? [
      "https://app.example.com",
      "https://admin.example.com"
    ] : ["*"]
    
    allowed_methods = ["GET", "POST", "PUT", "DELETE"]
    allowed_headers = ["Content-Type", "Authorization", "X-API-Key"]
    credentials     = true
    max_age         = 3600
  }
  
  # Rate limiting
  rate_limiting {
    enabled                = true
    requests_per_minute    = local.is_production ? 1000 : 10000
    burst_limit           = local.is_production ? 100 : 1000
    skip_successful_requests = false
  }
}

# Multiple database configurations
database "primary" {
  driver   = "postgresql"
  host     = var.db_host
  port     = var.db_port
  database = var.db_name
  username = var.db_username
  password = var.db_password
  
  pool {
    initial_size       = local.is_production ? 10 : 5
    max_size          = local.is_production ? 50 : 20
    min_idle          = local.is_production ? 5 : 2
    connection_timeout = 30
    idle_timeout      = 300
    max_lifetime      = 1200
  }
  
  ssl {
    mode    = local.is_production ? "require" : "prefer"
    ca_cert = local.is_production ? "/etc/ssl/certs/db-ca.crt" : null
  }
  
  read_only = false
}

database "replica" {
  driver   = "postgresql"
  host     = "${var.db_host}-replica"
  port     = var.db_port
  database = var.db_name
  username = "${var.db_username}_readonly"
  password = var.db_password
  
  pool {
    initial_size       = 5
    max_size          = 25
    min_idle          = 2
    connection_timeout = 30
    idle_timeout      = 300
    max_lifetime      = 1200
  }
  
  ssl {
    mode    = local.is_production ? "require" : "prefer"
    ca_cert = local.is_production ? "/etc/ssl/certs/db-ca.crt" : null
  }
  
  read_only = true
}

# Cache configuration with dynamic nodes
cache {
  provider = "redis"
  password = var.redis_password
  database = 0
  
  # Dynamic list of cache nodes
  dynamic "node" {
    for_each = var.redis_nodes
    
    content {
      host = node.value.host
      port = node.value.port
      role = node.value.role
    }
  }
  
  settings {
    default_ttl      = 3600
    max_memory       = local.is_production ? "1GB" : "256MB"
    eviction_policy  = "allkeys-lru"
    
    persistence {
      enabled  = local.is_production
      strategy = "rdb"
      interval = 900
    }
  }
}

# Monitoring configuration
monitoring {
  enabled = true
  
  metrics {
    enabled = true
    port    = 9090
    path    = "/metrics"
    
    collectors = [
      "cpu",
      "memory",
      "disk",
      "network",
      "application"
    ]
  }
  
  tracing {
    enabled       = local.is_production
    sampling_rate = local.is_production ? 0.1 : 1.0
    
    jaeger {
      endpoint     = "http://jaeger.example.com:14268"
      service_name = "payment-api"
      
      tags = {
        version     = "3.2.1"
        environment = local.environment
        datacenter  = var.datacenter
      }
    }
  }
  
  health_checks {
    enabled  = true
    endpoint = "/health"
    
    check {
      name    = "database"
      timeout = 5000
    }
    
    check {
      name    = "cache"
      timeout = 2000
    }
    
    check {
      name    = "external_api"
      timeout = 10000
    }
  }
}

# Feature flags with expressions
features = {
  payment_processing = true
  user_analytics     = true
  advanced_reporting = local.is_production
  debug_mode         = !local.is_production
  maintenance_mode   = false
  api_v2_enabled     = var.enable_api_v2
}

# Dynamic integration configurations
dynamic "integration" {
  for_each = var.integrations
  
  content {
    name         = integration.key
    type         = integration.value.type
    base_url     = integration.value.base_url
    api_key      = integration.value.api_key
    timeout      = integration.value.timeout
    retry_attempts = integration.value.retry_attempts
    
    health_check {
      enabled = lookup(integration.value, "health_check_enabled", true)
      path    = lookup(integration.value, "health_check_path", "/health")
    }
  }
}

# Output values for other configurations
output "database_url" {
  description = "Database connection URL"
  value       = local.database_url
  sensitive   = true
}

output "server_endpoint" {
  description = "Server endpoint URL"
  value       = "${local.server_config.ssl ? "https" : "http"}://${server.host}:${server.port}"
}

output "feature_flags" {
  description = "Enabled feature flags"
  value       = [for k, v in features : k if v]
}
```

<hr>
<a href="#top">&uarr; <b>TOP</b></a>
<br>
<br>


<!-- FOOT COPYRIGHT
################################################# -->
<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2025 <strong>JAMES GOBER.</strong></sup>
</div>

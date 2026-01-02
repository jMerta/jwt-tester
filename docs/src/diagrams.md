# Architectural Diagrams

## System Context

This diagram shows how `jwt-tester` fits into the user's environment.

```mermaid
graph TD
    User[User]
    CLI[jwt-tester CLI]
    UI[jwt-tester UI]
    Browser[Web Browser]
    VaultDB[(SQLite Vault)]
    Keychain[OS Keychain]

    User -- "Commands" --> CLI
    User -- "Interacts" --> Browser
    Browser -- "HTTP/JSON" --> UI
    
    CLI -- "Read/Write Metadata" --> VaultDB
    CLI -- "Read/Write Secrets" --> Keychain
    
    UI -- "Read/Write Metadata" --> VaultDB
    UI -- "Read/Write Secrets" --> Keychain
    
    subgraph "jwt-tester process"
        UI
        CLI
    end
```

## Verify Command Flow

How the tool resolves keys and verifies a token.

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Vault
    participant JWT
    
    User->>CLI: verify --project my-app <TOKEN>
    CLI->>JWT: Decode Header (get alg, kid)
    CLI->>Vault: Find project "my-app"
    
    alt Token has kid
        Vault->>Vault: Find key with matching kid
    else No kid
        Vault->>Vault: Check for default key
    end
    
    Vault-->>CLI: Return Candidate Key(s)
    
    loop For each Key
        CLI->>JWT: Verify Signature
        alt Valid
            CLI->>JWT: Validate Claims (exp, iss, aud)
            JWT-->>CLI: OK
            CLI-->>User: Success (Verified)
        else Invalid
            CLI-->>User: Error
        end
    end
```

## Vault Data Model

Relationship between entities.

```mermaid
erDiagram
    PROJECT ||--o{ KEY : contains
    PROJECT ||--o{ TOKEN : contains
    
    PROJECT {
        uuid id
        string name
        string description
        string default_key_id
    }
    
    KEY {
        uuid id
        uuid project_id
        string name
        string kind
        string kid
        string secret_ref "Stored in Keychain"
    }
    
    TOKEN {
        uuid id
        uuid project_id
        string name
        string token_ref "Stored in Keychain"
    }
```

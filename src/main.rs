use std::collections::HashMap;
use std::error::Error;
use std::fs::{self};
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Datelike};
use regex::Regex;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    cpf: String,
    full_name: String,
    email: String,
    birth: NaiveDate,
    role: UserRole,
}

type UserDatabase = Arc<Mutex<HashMap<String, User>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db: UserDatabase = Arc::new(Mutex::new(HashMap::new()));
    load_users_from_file(&db)?;

    loop {
        display_menu(&db).await?;
    }
}

fn load_users_from_file(db: &UserDatabase) -> Result<(), Box<dyn Error>> {
    if let Ok(file_content) = fs::read_to_string("users_data.txt") {
        let users: HashMap<String, User> = serde_json::from_str(&file_content)?;
        let mut db = db.lock().unwrap();
        *db = users;
    }
    Ok(())
}

fn save_users_to_file(db: &UserDatabase) -> Result<(), Box<dyn Error>> {
    let db = db.lock().unwrap();
    let file_content = serde_json::to_string(&*db)?;
    fs::write("users_data.txt", file_content)?;
    Ok(())
}

async fn display_menu(db: &UserDatabase) -> Result<(), Box<dyn Error>> {
    println!("Menu:");
    println!("1. Adicionar um novo usuário");
    println!("2. Atualizar um usuário existente");
    println!("3. Deletar um usuário");
    println!("4. Mostrar todos os usuários");
    println!("5. Sair");

    let mut choice = String::new();
    print!("Escolha uma opção: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut choice)?;

    match choice.trim() {
        "1" => {
            let user = input_user()?;
            let response = set_user(db, &user).await?;
            println!("Usuário criado com CPF: {:?}", response.cpf);
            save_users_to_file(db)?;
        }
        "2" => {
            let cpf = input_cpf()?;
            if let Ok(user) = get_user(db, &cpf).await {
                println!("Usuário encontrado: {:?}", user);
                let updated_user = input_user()?;
                let response = update_user(db, &cpf, &updated_user).await?;
                println!("Usuário atualizado: {:?}", response);
                save_users_to_file(db)?;
            } else {
                println!("Usuário não encontrado!");
            }
        }
        "3" => {
            let cpf = input_cpf()?;
            if delete_user(db, &cpf).await.is_ok() {
                println!("Usuário excluído!");
                save_users_to_file(db)?;
            } else {
                println!("Usuário não encontrado para exclusão!");
            }
        }
        "4" => {
            let users = get_users(db).await?;
            if users.is_empty() {
                println!("Nenhum usuário encontrado!");
            } else {
                println!("Todos os usuários:");
                for (cpf, user) in users {
                    println!(
                        "CPF: {}, Usuário: {:?}, Idade: {}",
                        cpf,
                        user,
                        calculate_age(&user.birth)
                    );
                }
            }
        }
        "5" => {
            println!("Saindo...");
            process::exit(0);
        }
        _ => {
            println!("Opção inválida, tente novamente!");
        }
    }

    Ok(())
}

fn input_user() -> Result<User, Box<dyn Error>> {
    let cpf = loop {
        let mut cpf = String::new();
        print!("CPF (11 dígitos numéricos): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut cpf)?;

        let cpf = cpf.trim().to_string();
        if Regex::new(r"^\d{11}$").unwrap().is_match(&cpf) {
            break cpf;
        } else {
            println!("O CPF deve conter apenas 11 dígitos numéricos! Tente novamente.");
        }
    };

    let full_name = loop {
        let mut full_name = String::new();
        print!("Nome completo (mínimo 10, máximo 100 caracteres): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut full_name)?;

        let full_name = full_name.trim().to_string();
        if full_name.len() >= 10 && full_name.len() <= 100 {
            break full_name;
        } else {
            println!("O nome completo deve ter entre 10 e 100 caracteres! Tente novamente.");
        }
    };

    let email = loop {
        let mut email = String::new();
        print!("Email: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut email)?;

        let email = email.trim().to_string();
        if !email.contains(' ')
            && Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.(com|br)$").unwrap().is_match(&email)
            && email.len() >= 15
            && email.len() <= 50
        {
            break email;
        } else {
            println!("O email deve ser válido, sem espaços, terminar com .com ou .br, e ter entre 15 e 50 caracteres! Tente novamente.");
        }
    };

    let birth = loop {
        let mut birth = String::new();
        print!("Data de nascimento (DD-MM-YYYY): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut birth)?;

        match NaiveDate::parse_from_str(birth.trim(), "%d-%m-%Y") {
            Ok(date) if date.year() >= 1909 && date.year() <= 2024 => break date,
            _ => println!("Data inválida! Use o formato DD-MM-YYYY e verifique o ano. Tente novamente."),
        }
    };

    let role = loop {
        let mut role = String::new();
        print!("Cargo (Admin/User/Guest): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut role)?;

        match role.trim().to_lowercase().as_str() {
            "admin" => break UserRole::Admin,
            "user" => break UserRole::User,
            "guest" => break UserRole::Guest,
            _ => println!("Cargo inválido! Escolha entre Admin, User ou Guest. Tente novamente."),
        }
    };

    Ok(User {
        cpf,
        full_name,
        email,
        birth,
        role,
    })
}


fn input_cpf() -> Result<String, Box<dyn Error>> {
    let mut cpf = String::new();
    print!("CPF do usuário: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut cpf)?;
    Ok(cpf.trim().to_string())
}

fn calculate_age(birth_date: &NaiveDate) -> u32 {
    let today = chrono::Local::now().naive_utc().date();
    let mut age = today.year() - birth_date.year();

    if (today.month() < birth_date.month())
        || (today.month() == birth_date.month() && today.day() < birth_date.day()) {
        age -= 1;
    }

    age as u32
}

async fn set_user(db: &UserDatabase, user: &User) -> Result<User, Box<dyn Error>> {
    let mut db = db.lock().unwrap();

    if db.contains_key(&user.cpf) {
        return Err("CPF já cadastrado!".into());
    }

    let user_clone = user.clone();
    db.insert(user.cpf.clone(), user_clone);
    Ok(user.clone())
}

async fn get_users(db: &UserDatabase) -> Result<HashMap<String, User>, Box<dyn Error>> {
    let db = db.lock().unwrap();
    Ok(db.clone())
}

async fn get_user(db: &UserDatabase, cpf: &str) -> Result<User, Box<dyn Error>> {
    let db = db.lock().unwrap();
    if let Some(user) = db.get(cpf) {
        Ok(user.clone())
    } else {
        Err("Usuário não encontrado!".into())
    }
}

async fn update_user(db: &UserDatabase, cpf: &str, new_user: &User) -> Result<User, Box<dyn Error>> {
    let mut db = db.lock().unwrap();

    if let Some(user) = db.get_mut(cpf) {
        *user = new_user.clone();
        Ok(user.clone())
    } else {
        Err("Usuário não encontrado!".into())
    }
}

async fn delete_user(db: &UserDatabase, cpf: &str) -> Result<(), Box<dyn Error>> {
    let mut db = db.lock().unwrap();

    if db.remove(cpf).is_none() {
        Err("Usuário não encontrado!".into())
    } else {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use chrono::NaiveDate;

    fn mock_database() -> UserDatabase {
        Arc::new(Mutex::new(HashMap::new()))
    }

    fn mock_user(cpf: &str) -> User {
        User {
            cpf: cpf.to_string(),
            full_name: "Test User".to_string(),
            email: "testuser@example.com".to_string(),
            birth: NaiveDate::from_ymd_opt(1995, 5, 15)
            .expect("Data inválida fornecida para 'birth'"),
            role: UserRole::User,
        }
    }

    #[tokio::test]
    async fn test_set_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        
        let result = set_user(&db, &user).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cpf, "12345678901");
    }

    #[tokio::test]
    async fn test_set_user_duplicate_cpf() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await; 
        
        let duplicate_result = set_user(&db, &user).await; 
        assert!(duplicate_result.is_err());
        assert_eq!(
            duplicate_result.unwrap_err().to_string(),
            "CPF já cadastrado!"
        );
    }

    #[tokio::test]
    async fn test_get_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await;

        let fetched_user = get_user(&db, "12345678901").await;
        assert!(fetched_user.is_ok());
        assert_eq!(fetched_user.unwrap().cpf, "12345678901");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let db = mock_database();
        let fetched_user = get_user(&db, "99999999999").await;
        assert!(fetched_user.is_err());
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await;

        let delete_result = delete_user(&db, "12345678901").await;
        assert!(delete_result.is_ok());

        let fetch_result = get_user(&db, "12345678901").await;
        assert!(fetch_result.is_err());
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await;

        // Cria o usuário atualizado
        let updated_user = User {
            cpf: "12345678901".to_string(),
            full_name: "Updated User".to_string(),
            email: "updateduser@example.com".to_string(),
            birth: NaiveDate::from_ymd_opt(1995, 5, 15)
                .expect("Data inválida fornecida para 'birth'"),
            role: UserRole::Admin,
        };

        let update_result = update_user(&db, "12345678901", &updated_user).await;
        assert!(update_result.is_ok());

        // Verifica se o usuário foi atualizado
        let fetched_user = get_user(&db, "12345678901").await;
        assert!(fetched_user.is_ok());
        let fetched_user = fetched_user.unwrap();
        
        assert_eq!(fetched_user.cpf, "12345678901");
        assert_eq!(fetched_user.full_name, "Updated User");
        assert_eq!(fetched_user.email, "updateduser@example.com");
        assert_eq!(fetched_user.role, UserRole::Admin);
    }
}

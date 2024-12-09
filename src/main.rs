use std::collections::HashMap;
use std::error::Error;
use std::fs::{self};
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Datelike};
use regex::Regex;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    println!("3. Criar um novo usuário");
    println!("4. Deletar um usuário");
    println!("5. Mostrar todos os usuários");
    println!("6. Sair");

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
            let user = input_user()?;
            let response = set_user(db, &user).await?;
            println!("Usuário criado com CPF: {:?}", response.cpf);
            save_users_to_file(db)?;
        }
        "4" => {
            let cpf = input_cpf()?;
            if delete_user(db, &cpf).await.is_ok() {
                println!("Usuário excluído!");
                save_users_to_file(db)?;
            } else {
                println!("Usuário não encontrado para exclusão!");
            }
        }
        "5" => {
            let users = get_users(db).await?;
            if users.is_empty() {
                println!("Nenhum usuário encontrado!");
            } else {
                println!("Todos os usuários:");
                for (cpf, user) in users {
                    println!("CPF: {}, Usuário: {:?}, Idade: {}", cpf, user, calculate_age(&user.birth));
                }
            }
        }
        "6" => {
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
    let mut cpf = String::new();
    let mut full_name = String::new();
    let mut email = String::new();
    let mut birth = String::new();
    let mut role = String::new();

    print!("CPF (11 dígitos numéricos): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut cpf)?;

    let cpf = cpf.trim().to_string();
    if !Regex::new(r"^\d{11}$").unwrap().is_match(&cpf) {
        return Err("O CPF deve conter apenas 11 dígitos numéricos!".into());
    }

    print!("Nome completo (mínimo 10, máximo 100 caracteres): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut full_name)?;

    let full_name = full_name.trim().to_string();
    if full_name.len() < 10 || full_name.len() > 100 {
        return Err("O nome completo deve ter entre 10 e 100 caracteres!".into());
    }

    print!("Email: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut email)?;

    let email = email.trim().to_string();
    if email.contains(' ') {
        return Err("O email não pode conter espaços!".into());
    }

    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.(com|br)$").unwrap();
    if !email_regex.is_match(&email) {
        return Err("O email deve terminar com .com ou .br!".into());
    }
    if email.len() < 15 || email.len() > 50 {
        return Err("O email deve ter entre 15 e 50 caracteres!".into());
    }

    print!("Data de nascimento (DD-MM-YYYY): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut birth)?;

    let birth_date = NaiveDate::parse_from_str(birth.trim(), "%d-%m-%Y");
    match birth_date {
        Ok(date) => {
            if date.year() < 1909 || date.year() > 2024 {
                return Err("O ano deve ser maior que 1909 e menor que 2024!".into());
            }
            if date.month() < 1 || date.month() > 12 {
                return Err("O mês deve estar entre 1 e 12!".into());
            }
            if date.day() < 1 || date.day() > 31 {
                return Err("O dia deve estar entre 1 e 31!".into());
            }
        }
        Err(_) => return Err("Data inválida! Use o formato DD-MM-YYYY.".into()),
    }

    print!("Cargo (Admin/User/Guest): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut role)?;

    let role = match role.trim().to_lowercase().as_str() {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        "guest" => UserRole::Guest,
        _ => return Err("Cargo inválido!".into()),
    };

    Ok(User {
        cpf,
        full_name,
        email,
        birth: birth_date.unwrap(),
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
        if new_user.cpf != user.cpf {
            return Err("O CPF não pode ser alterado!".into());
        }
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
